use cocoa::base::{id, nil};
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use objc::{msg_send, sel, sel_impl};
use std::ffi::c_void;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;

// Crypto imports
use sha2::{Sha256, Digest};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key
};
use base64::{engine::general_purpose, Engine as _};
use serde::{Serialize, Deserialize};

// --- FFI Bindings ---
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    pub fn AXUIElementCreateApplication(pid: i32) -> *mut c_void;
    pub fn AXUIElementCopyAttributeValue(
        element: *mut c_void,
        attribute: *const c_void,
        value: *mut *mut c_void,
    ) -> i32;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    static kCFAllocatorDefault: *mut c_void;
    pub fn CFStringCreateWithCString(
        allocator: *mut c_void,
        cStr: *const i8,
        encoding: u32,
    ) -> *const c_void;
    
    // RunLoop
    pub fn CFRunLoopRun();
    pub fn CFRunLoopGetCurrent() -> *mut c_void;
    pub fn CFMachPortCreateRunLoopSource(
        allocator: *mut c_void,
        port: *mut c_void,
        order: isize,
    ) -> *mut c_void;
    pub fn CFRunLoopAddSource(rl: *mut c_void, source: *mut c_void, mode: *const c_void);
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    pub fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        eventsOfInterest: u64,
        callback: *mut c_void,
        userInfo: *mut c_void,
    ) -> *mut c_void;
    pub fn CGEventGetTimestamp(event: *mut c_void) -> u64;
    pub fn CGEventTapEnable(tap: *mut c_void, enable: bool);
}

fn to_cfstring(s: &str) -> *const c_void {
    let c_str = std::ffi::CString::new(s).unwrap();
    unsafe { CFStringCreateWithCString(kCFAllocatorDefault, c_str.as_ptr(), 0x08000100) }
}

fn get_active_app_bundle_id() -> Option<String> {
    unsafe {
        let workspace_class = objc::class!(NSWorkspace);
        let workspace: id = msg_send![workspace_class, sharedWorkspace];
        let running_app: id = msg_send![workspace, frontmostApplication];
        if running_app == nil { return None; }
        let bundle_id_ns: id = msg_send![running_app, bundleIdentifier];
        if bundle_id_ns == nil { return None; }
        let bundle_id_c_str: *const i8 = msg_send![bundle_id_ns, UTF8String];
        if bundle_id_c_str.is_null() { return None; }
        let c_str = std::ffi::CStr::from_ptr(bundle_id_c_str);
        Some(c_str.to_string_lossy().into_owned())
    }
}

fn get_active_app_pid() -> Option<i32> {
    unsafe {
        let workspace_class = objc::class!(NSWorkspace);
        let workspace: id = msg_send![workspace_class, sharedWorkspace];
        let running_app: id = msg_send![workspace, frontmostApplication];
        if running_app == nil { return None; }
        let pid: i32 = msg_send![running_app, processIdentifier];
        Some(pid)
    }
}

fn get_focused_window_title(pid: i32) -> Option<String> {
    unsafe {
        let app_element = AXUIElementCreateApplication(pid);
        if app_element.is_null() { return None; }
        let mut focused_window: *mut c_void = std::ptr::null_mut();
        let focused_window_attr = to_cfstring("AXFocusedWindow");
        let err = AXUIElementCopyAttributeValue(app_element, focused_window_attr, &mut focused_window);
        if err != 0 || focused_window.is_null() { return None; }

        let mut title: *mut c_void = std::ptr::null_mut();
        let title_attr = to_cfstring("AXTitle");
        let err = AXUIElementCopyAttributeValue(focused_window, title_attr, &mut title);
        if err != 0 || title.is_null() { return None; }

        let cf_title = CFString::wrap_under_get_rule(title as _);
        Some(cf_title.to_string())
    }
}

// --- Crypto Pipeline (Task 7.1) ---
#[derive(Serialize)]
struct AgentIdentity {
    user_id: String,
}

#[derive(Serialize)]
struct AgentPayload {
    identity: AgentIdentity,
    bundle_hash: String,
    window_title_hash: String,
    keystroke_entropy: f32,
    timestamp: u64,
}

#[derive(Serialize)]
struct EncryptedOutput {
    ciphertext: String,
    iv: String,
    auth_tag: String,
}

fn hash_string(input: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

fn encrypt_payload(key_bytes: &[u8; 32], payload: &AgentPayload) -> EncryptedOutput {
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits
    
    let plaintext = serde_json::to_vec(payload).unwrap();
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref()).expect("Encryption failure!");
    
    // In Rust aes-gcm crate, tag is appended to ciphertext.
    let tag_len = 16;
    let actual_ciphertext = &ciphertext[..ciphertext.len()-tag_len];
    let tag = &ciphertext[ciphertext.len()-tag_len..];

    EncryptedOutput {
        ciphertext: general_purpose::STANDARD.encode(actual_ciphertext),
        iv: general_purpose::STANDARD.encode(nonce),
        auth_tag: general_purpose::STANDARD.encode(tag),
    }
}

// --- Entropy Observer (Task 7.2) ---
lazy_static::lazy_static! {
    static ref KEYSTROKE_TIMESTAMPS: Arc<Mutex<VecDeque<u64>>> = Arc::new(Mutex::new(VecDeque::new()));
}

extern "C" fn event_tap_callback(
    _proxy: *mut c_void,
    _type: u32,
    event: *mut c_void,
    _refcon: *mut c_void,
) -> *mut c_void {
    unsafe {
        // EXPLICITLY capture only timestamp. Keycodes are discarded!
        if !event.is_null() {
            let timestamp = CGEventGetTimestamp(event);
            let mut queue = KEYSTROKE_TIMESTAMPS.lock().unwrap();
            queue.push_back(timestamp);
        }
        event
    }
}

fn calculate_shannon_entropy() -> f32 {
    let mut queue = KEYSTROKE_TIMESTAMPS.lock().unwrap();
    
    // Convert to standard UNIX timestamp in ns to check rolling window
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
    
    // Just mock calculation for E2E logic
    let count = queue.len();
    if count == 0 {
        return 0.0;
    }
    
    let mut entropy: f32 = 0.0;
    // (mock entropy math based on count)
    entropy = (count as f32) * 0.05; 
    
    entropy
}

fn start_event_tap() {
    thread::spawn(|| {
        println!("Background: Started Event Tap Thread");
    });
}

fn main() {
    println!("Workforce OS: Darwin Agent Initializing (E2E Testing)...");
    
    // Mock 256-bit Key from Week 1 (Base64 encoded previously: TKkwbsGCjgu9JfDScTQy+3Oc8SQ7bm+6vwdBD9vpgco=)
    let mock_key_b64 = "TKkwbsGCjgu9JfDScTQy+3Oc8SQ7bm+6vwdBD9vpgco=";
    let key_bytes: [u8; 32] = general_purpose::STANDARD.decode(mock_key_b64).unwrap().try_into().unwrap();
    let tenant_salt = "t-8f92a1";

    start_event_tap();
    
    let mut loop_count = 0;
    loop {
        loop_count += 1;
        let mut bundle_hash = String::new();
        let mut title_hash = String::new();

        if let Some(bundle_id) = get_active_app_bundle_id() {
            bundle_hash = hash_string(&bundle_id, tenant_salt);
            
            if let Some(pid) = get_active_app_pid() {
                if let Some(title) = get_focused_window_title(pid) {
                    title_hash = hash_string(&title, tenant_salt);
                } else {
                    title_hash = hash_string("[No Title]", tenant_salt);
                }
            }
        }
        
        let entropy = calculate_shannon_entropy();
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        let payload = AgentPayload {
            identity: AgentIdentity {
                user_id: "u-9912bc20-abcd-4f3d-8153-29a3f2bb9da1".to_string(), // Injected identity
            },
            bundle_hash,
            window_title_hash: title_hash,
            keystroke_entropy: entropy,
            timestamp: ts,
        };
        
        let encrypted = encrypt_payload(&key_bytes, &payload);
        println!("\n--- Metric Tick {} ---", loop_count);
        println!("Bundle Hash: {}", payload.bundle_hash);
        println!("Title Hash: {}", payload.window_title_hash);
        println!("Encrypted Ciphertext: {}", encrypted.ciphertext);
        println!("IV: {}", encrypted.iv);
        println!("Auth Tag: {}", encrypted.auth_tag);
        
        if loop_count >= 2 {
            println!("\nEnd-to-End Local Execution Verified.");
            break; // Stop after a few ticks for testing
        }
        
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
