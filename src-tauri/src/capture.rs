use screenshots::Screen;
use image::{DynamicImage, ImageOutputFormat};
use std::thread;
use std::time::Duration;
use std::io::Cursor;
use base64::{Engine as _, engine::general_purpose};
use crate::stitch;
use tauri::{AppHandle, Emitter, Manager};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use lazy_static::lazy_static;
// use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState}; // Removed

lazy_static! {
    static ref CAPTURE_STATES: Mutex<HashMap<String, Arc<Mutex<bool>>>> = Mutex::new(HashMap::new());
}

#[tauri::command]
pub async fn start_scroll_capture(app: AppHandle, x: i32, y: i32, width: u32, height: u32) -> Result<(), String> {
    println!("Starting manual scroll capture task at ({}, {}) {}x{}", x, y, width, height);
    
    // Instead of hiding, we set ignore cursor events to true
    // This allows the window to remain visible (showing the green border) but let clicks pass through
    let windows = app.webview_windows();
    for (label, window) in windows {
        println!("Setting ignore cursor events for window: {}", label);
        let _ = window.set_ignore_cursor_events(true);
    }
    
    // Give the window manager some time to update
    thread::sleep(Duration::from_millis(200));

    // Create a stop flag for this capture session
    let stop_flag = Arc::new(Mutex::new(false));
    let stop_flag_clone = stop_flag.clone();
    
    // Store it so we can access it from stop command
    // We use a simple key "current" since we only allow one capture at a time
    CAPTURE_STATES.lock().unwrap().insert("current".to_string(), stop_flag);
    
    // Start listening for F9 using rdev in a separate thread
    let stop_flag_listener = stop_flag_clone.clone();
    std::thread::spawn(move || {
        use rdev::{listen, Event, EventType, Key};
        
        let callback = move |event: Event| {
            if let EventType::KeyPress(Key::Escape) = event.event_type {
                println!("Esc detected via rdev, stopping capture...");
                let mut stop = stop_flag_listener.lock().unwrap();
                *stop = true;
                // Force exit the process if needed? No, let the loop break gracefully.
            }
        };
        
        // This will block until error
        // Note: rdev listen loop is blocking.
        // We run it in a thread so it doesn't block the UI or capture loop.
        if let Err(error) = listen(callback) {
            println!("Error: {:?}", error);
        }
    });

    // Spawn a thread to handle the long-running capture process
    std::thread::spawn(move || {
        let result = run_capture_loop(&app, x, y, width, height, stop_flag_clone);
        if let Err(e) = result {
            println!("Capture loop error: {}", e);
            let _ = app.emit("capture-error", e);
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_scroll_capture() -> Result<(), String> {
    println!("Stopping capture...");
    if let Some(flag) = CAPTURE_STATES.lock().unwrap().get("current") {
        let mut stop = flag.lock().unwrap();
        *stop = true;
    }
    Ok(())
}

fn run_capture_loop(app: &AppHandle, x: i32, y: i32, width: u32, height: u32, stop_flag: Arc<Mutex<bool>>) -> Result<(), String> {
    // 1. Initial Capture
    // Shrink the capture area slightly (2px on each side) to avoid capturing the green border
    let border_offset = 2;
    let safe_x = x + border_offset;
    let safe_y = y + border_offset;
    let safe_width = if width > (border_offset as u32 * 2) { width - (border_offset as u32 * 2) } else { width };
    let safe_height = if height > (border_offset as u32 * 2) { height - (border_offset as u32 * 2) } else { height };

    let mut full_image = capture_rect(safe_x, safe_y, safe_width, safe_height).map_err(|e| e.to_string())?;
    
    // Allow up to 500 stitches (very long image)
    let max_stitches = 500; 
    let mut stitch_count = 0;

    println!("Entering capture loop. Please scroll manually.");

    loop {
        // Check stop flag
        {
            let stop = stop_flag.lock().unwrap();
            if *stop {
                println!("Stop flag detected. Finishing capture.");
                break;
            }
        }

        if stitch_count >= max_stitches {
            println!("Reached max stitches limit.");
            break;
        }
        
        // 2. Wait a bit for user to scroll
        thread::sleep(Duration::from_millis(100));
        
        // 3. Capture new fragment
        // We capture without hiding the window, because we are capturing inside the border
        let new_fragment = match capture_rect(safe_x, safe_y, safe_width, safe_height) {
            Ok(img) => img,
            Err(e) => {
                println!("Capture failed: {}", e);
                break;
            }
        };
        
        // 4. Calculate overlap
        let overlap_index = stitch::calculate_overlap(&full_image, &new_fragment);
        
        // Check for static content (identical image)
        if overlap_index == new_fragment.height() - 1 {
            // Just continue loop, waiting for user to scroll or stop
            continue;
        }
        
        // Check for no overlap (too fast or error)
        if overlap_index == 0 {
             continue;
        }
        
        println!("Stitching: overlap index {}", overlap_index);

        // 5. Stitch
        full_image = stitch::append_image(&full_image, &new_fragment, overlap_index);
        stitch_count += 1;
    }
    
    println!("Capture finished. Total height: {}", full_image.height());
    
    // Convert to Base64
    let base64_img = image_to_base64(&full_image).map_err(|e| e.to_string())?;
    
    // Re-enable cursor events for ALL windows before showing them
    let windows = app.webview_windows();
    for (label, window) in windows {
        println!("Restoring cursor events for window: {}", label);
        let _ = window.set_ignore_cursor_events(false);
        let _ = window.show();
        let _ = window.set_focus();
    }

    // Emit event with result
    app.emit("capture-complete", base64_img).map_err(|e| e.to_string())?;
    
    Ok(())
}

fn toggle_window_visibility(app: &AppHandle, visible: bool) {
    let windows = app.webview_windows();
    for (_label, window) in windows {
        if visible {
            let _ = window.show();
        } else {
            let _ = window.hide();
        }
    }
}

fn capture_rect(x: i32, y: i32, width: u32, height: u32) -> Result<DynamicImage, String> {
    let screens = Screen::all().map_err(|e| format!("Failed to get screens: {}", e))?;
    
    // Find the screen that contains the point (x, y)
    // We assume x, y are Global Physical Coordinates
    let screen = screens.iter().find(|s| {
        let sx = s.display_info.x;
        let sy = s.display_info.y;
        let sw = s.display_info.width;
        let sh = s.display_info.height;
        
        // Check if the center of the rect is within this screen
        let cx = x + (width as i32 / 2);
        let cy = y + (height as i32 / 2);
        
        cx >= sx && cx < sx + sw as i32 && cy >= sy && cy < sy + sh as i32
    }).or(screens.first()).ok_or("No screen found")?;

    // Calculate relative coordinates within the screen
    // Since x, y are already physical, we just subtract the screen's physical origin
    let rx = x - screen.display_info.x;
    let ry = y - screen.display_info.y;
    
    // Width and height are also physical
    let rw = width;
    let rh = height;

    let image = screen.capture_area(rx, ry, rw, rh)
        .map_err(|e| format!("Failed to capture area: {}", e))?;
        
    Ok(DynamicImage::ImageRgba8(image))
}

fn image_to_base64(img: &DynamicImage) -> Result<String, String> {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageOutputFormat::Png)
        .map_err(|e| format!("Failed to encode image: {}", e))?;
        
    let res_base64 = general_purpose::STANDARD.encode(buf.into_inner());
    Ok(format!("data:image/png;base64,{}", res_base64))
}
