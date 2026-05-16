mod context;
mod accessibility;

use std::thread;
use std::time::Duration;

fn main() {
    println!("[SYSTEM] Darwin Telemetry Agent Initialized.");
    println!("[SYSTEM] Target: aarch64-apple-darwin.");
    
    let mut loop_count = 0;
    loop {
        if let Some(bundle) = context::get_active_app_bundle() {
            let mut title = "[No Title or AX Permission Denied]".to_string();
            
            if let Some(pid) = context::get_active_app_pid() {
                if let Some(focused_title) = accessibility::get_focused_window_title(pid) {
                    title = focused_title;
                }
            }
            
            // ACTION REQUIRED: Route this data to the Week 1 Masking Pipeline
            // mask_string(title, tenant_salt)
            
            println!("[DEBUG] Active: {} | Title Captured (Pre-Hash): {}", bundle, title);
        }
        
        loop_count += 1;
        if loop_count >= 2 {
            println!("\n[SYSTEM] Agent logic locally verified. Exiting.");
            break;
        }
        
        thread::sleep(Duration::from_secs(5));
    }
}
