// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Fix for Linux WebKit rendering issues (inverted/mirrored UI) in VMs or specific drivers
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    scroll_snap_lib::run()
}
