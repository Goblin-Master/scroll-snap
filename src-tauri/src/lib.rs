use tauri::Builder;

pub mod capture;
pub mod stitch;
pub mod utils;

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
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(|app, shortcut, event| {
            println!("Shortcut pressed: {:?}", shortcut);
            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed  {
                if shortcut.matches(tauri_plugin_global_shortcut::Shortcut::parse("F9").unwrap()) {
                    // Call stop capture
                    println!("F9 pressed, stopping capture...");
                    let _ = tauri::async_runtime::block_on(async {
                        capture::stop_scroll_capture().await
                    });
                }
            }
        }).build())
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
