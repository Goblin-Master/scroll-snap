use tauri::Manager;

mod capture;
mod stitch;
mod utils;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet, 
            capture::start_scroll_capture,
            capture::stop_scroll_capture,
            utils::copy_to_clipboard,
            utils::save_image
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
