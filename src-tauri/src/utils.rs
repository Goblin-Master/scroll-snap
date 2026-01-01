use arboard::Clipboard;
use image::load_from_memory;
use base64::{Engine as _, engine::general_purpose};
use std::borrow::Cow;

#[tauri::command]
pub fn copy_to_clipboard(base64_image: String) -> Result<(), String> {
    // Remove header if present
    let b64 = base64_image.trim_start_matches("data:image/png;base64,");
    let bytes = general_purpose::STANDARD.decode(b64)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;
        
    let img = load_from_memory(&bytes)
        .map_err(|e| format!("Failed to load image: {}", e))?;
    
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let image_data = arboard::ImageData {
        width: w as usize,
        height: h as usize,
        bytes: Cow::from(rgba.into_raw()),
    };
    
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_image(image_data).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub fn save_image(path: String, base64_image: String) -> Result<(), String> {
    use std::fs::File;
    use std::io::Write;
    
    let b64 = base64_image.trim_start_matches("data:image/png;base64,");
    let bytes = general_purpose::STANDARD.decode(b64)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;
        
    let mut file = File::create(path).map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;
    
    Ok(())
}
