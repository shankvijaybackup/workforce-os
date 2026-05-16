use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use std::ffi::c_void;

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
}

fn to_cfstring(s: &str) -> *const c_void {
    let c_str = std::ffi::CString::new(s).unwrap();
    unsafe { CFStringCreateWithCString(kCFAllocatorDefault, c_str.as_ptr(), 0x08000100) } // kCFStringEncodingUTF8
}

pub fn get_focused_window_title(pid: i32) -> Option<String> {
    unsafe {
        let app_element = AXUIElementCreateApplication(pid);
        if app_element.is_null() {
            return None;
        }

        let mut focused_window: *mut c_void = std::ptr::null_mut();
        let focused_window_attr = to_cfstring("AXFocusedWindow");
        
        let err = AXUIElementCopyAttributeValue(app_element, focused_window_attr, &mut focused_window);
        if err != 0 || focused_window.is_null() {
            return None;
        }

        let mut title: *mut c_void = std::ptr::null_mut();
        let title_attr = to_cfstring("AXTitle");
        let err = AXUIElementCopyAttributeValue(focused_window, title_attr, &mut title);
        
        if err != 0 || title.is_null() {
            return None;
        }

        let cf_title = CFString::wrap_under_get_rule(title as _);
        Some(cf_title.to_string())
    }
}
