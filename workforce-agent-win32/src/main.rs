use std::thread;
use windows::Win32::Foundation::{CloseHandle, HMODULE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, GetWindowTextW,
    GetWindowThreadProcessId, SetWindowsHookExW, UnhookWindowsHookEx,
    EVENT_SYSTEM_FOREGROUND, HHOOK, KBDLLHOOKSTRUCT, MSG, WINEVENT_OUTOFCONTEXT,
    WH_KEYBOARD_LL,
};
use windows::Win32::UI::Accessibility::{SetWinEventHook, HWINEVENTHOOK};

use crossbeam_channel::{bounded, Sender, Receiver};

mod cache;
mod service;

// Masking logic
use sha2::{Digest, Sha256};
fn mask_string(input: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

lazy_static::lazy_static! {
    // Global bounded channel for lock-free, zero-latency input spooling.
    static ref KEYSTROKE_CHANNEL: (Sender<u32>, Receiver<u32>) = bounded(10000);
}

// --- Task 8.2: Zero-Intrusion Context Capture (Foreground Hook) ---
unsafe extern "system" fn win_event_proc(
    _h_win_event_hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    if hwnd.0 == 0 {
        return;
    }

    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));

    let mut process_name = String::from("Unknown");
    if let Ok(process_handle) = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid) {
        let mut buffer = [0u16; 1024];
        let len = GetModuleFileNameExW(process_handle, HMODULE::default(), &mut buffer);
        if len > 0 {
            process_name = String::from_utf16_lossy(&buffer[..len as usize]);
        }
        // DEFECT 1 RESOLVED: Explicitly close the kernel handle to prevent resource exhaustion.
        let _ = CloseHandle(process_handle);
    }

    let mut title_buffer = [0u16; 1024];
    let len = GetWindowTextW(hwnd, &mut title_buffer);
    let window_title = if len > 0 {
        String::from_utf16_lossy(&title_buffer[..len as usize])
    } else {
        String::from("[No Title]")
    };

    let tenant_salt = "t-8f92a1";
    let masked_process = mask_string(&process_name, tenant_salt);
    let masked_title = mask_string(&window_title, tenant_salt);

    // DEFECT 3 RESOLVED: Unmasked strings dropped immediately. Debug output only records hashes.
    println!(
        "[WIN32 EVENT] Active (Masked): {} | Title (Masked): {}",
        masked_process, masked_title
    );
}

// --- Task 8.3: Keystroke Entropy Engine (Low-Level Hook) ---
unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let hook_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
        let timestamp = hook_struct.time;
        
        // DEFECT 2 RESOLVED: Lock-free try_send prevents UI thread blocking and mutex contention.
        let _ = KEYSTROKE_CHANNEL.0.try_send(timestamp);
    }
    
    CallNextHookEx(HHOOK::default(), n_code, w_param, l_param)
}

fn start_entropy_worker() {
    thread::spawn(|| {
        let receiver = &KEYSTROKE_CHANNEL.1;
        let mut timestamps = std::collections::VecDeque::new();
        
        // Background thread safely processes the timestamps
        loop {
            if let Ok(ts) = receiver.recv() {
                timestamps.push_back(ts);
                // Manage 60-second rolling window here (mocked for architectural review)
                // if ts - timestamps.front().unwrap() > 60_000 { timestamps.pop_front(); }
                // let entropy = calculate_shannon_entropy(&timestamps);
            }
        }
    });
}

pub fn start_agent_loop() {
    unsafe {
        let _event_hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            HMODULE::default(),
            Some(win_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );

        let h_mod = GetModuleHandleW(None).unwrap_or_default();
        let keyboard_hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(low_level_keyboard_proc),
            h_mod,
            0,
        ).expect("Failed to install WH_KEYBOARD_LL hook");

        println!("[SYSTEM] Hooks registered successfully. Entering WinMain message loop...");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND::default(), 0, 0).into() {
            DispatchMessageW(&msg);
        }

        UnhookWindowsHookEx(keyboard_hook).ok();
    }
}

fn main() {
    println!("[SYSTEM] Win32 Telemetry Agent Initialized.");
    println!("[SYSTEM] Target: x86_64-pc-windows-msvc.");

    // Initialize Cache
    if let Ok(_telemetry_cache) = cache::TelemetryCache::new() {
        println!("[SYSTEM] Offline WAL cache initialized successfully.");
    } else {
        println!("[SYSTEM] Failed to initialize WAL cache.");
    }

    start_entropy_worker();

    // Start as a Windows Service. If running interactively, this may fail, 
    // so we fallback to start_agent_loop() for local debugging.
    if let Err(_) = service::run_as_service() {
        println!("[SYSTEM] Not running as a service, falling back to console loop.");
        start_agent_loop();
    }
}
