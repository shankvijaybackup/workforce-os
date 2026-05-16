use cocoa::base::{id, nil};
use objc::{msg_send, sel, sel_impl};
use core_foundation::base::TCFType;

pub fn get_active_app_bundle() -> Option<String> {
    unsafe {
        let workspace_class = objc::class!(NSWorkspace);
        let workspace: id = msg_send![workspace_class, sharedWorkspace];
        let active_app: id = msg_send![workspace, frontmostApplication];
        
        if active_app == nil {
            return None;
        }

        let bundle_id: id = msg_send![active_app, bundleIdentifier];
        if bundle_id == nil {
            return None;
        }

        let c_str: *const i8 = msg_send![bundle_id, UTF8String];
        if c_str.is_null() {
            return None;
        }
        
        let bundle_str = std::ffi::CStr::from_ptr(c_str).to_string_lossy().into_owned();
        Some(bundle_str)
    }
}

pub fn get_active_app_pid() -> Option<i32> {
    unsafe {
        let workspace_class = objc::class!(NSWorkspace);
        let workspace: id = msg_send![workspace_class, sharedWorkspace];
        let running_app: id = msg_send![workspace, frontmostApplication];
        if running_app == nil { return None; }
        let pid: i32 = msg_send![running_app, processIdentifier];
        Some(pid)
    }
}
