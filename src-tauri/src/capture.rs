use screenshots::Screen;
use image::{DynamicImage, ImageOutputFormat};
use std::thread;
use std::time::Duration;
use std::io::Cursor;
use base64::{Engine as _, engine::general_purpose};
use crate::stitch;
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn start_scroll_capture(app: AppHandle, x: i32, y: i32, width: u32, height: u32) -> Result<(), String> {
    println!("Starting manual scroll capture task at ({}, {}) {}x{}", x, y, width, height);
    
    // Spawn a thread to handle the long-running capture process
    std::thread::spawn(move || {
        let result = run_capture_loop(&app, x, y, width, height);
        if let Err(e) = result {
            println!("Capture loop error: {}", e);
            let _ = app.emit("capture-error", e);
        }
    });

    Ok(())
}

fn run_capture_loop(app: &AppHandle, x: i32, y: i32, width: u32, height: u32) -> Result<(), String> {
    // 1. Initial Capture
    let mut full_image = capture_rect(x, y, width, height).map_err(|e| e.to_string())?;
    
    let mut static_count = 0;
    let max_static_count = 30; // 3 seconds (30 * 100ms)
    
    // Allow up to 500 stitches (very long image)
    let max_stitches = 500; 
    let mut stitch_count = 0;

    println!("Entering capture loop. Please scroll manually.");

    loop {
        if stitch_count >= max_stitches {
            println!("Reached max stitches limit.");
            break;
        }
        
        // 2. Wait a bit for user to scroll
        thread::sleep(Duration::from_millis(100));
        
        // 3. Capture new fragment
        let new_fragment = match capture_rect(x, y, width, height) {
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
            static_count += 1;
            if static_count >= max_static_count {
                println!("Static content detected for {}s. Stopping.", max_static_count as f32 * 0.1);
                break;
            }
            continue;
        }
        
        // Check for no overlap (too fast or error)
        if overlap_index == 0 {
             static_count += 1;
             if static_count >= max_static_count {
                break;
             }
             continue;
        }
        
        // Reset static count since we found valid movement
        static_count = 0;
        
        println!("Stitching: overlap index {}", overlap_index);

        // 5. Stitch
        full_image = stitch::append_image(&full_image, &new_fragment, overlap_index);
        stitch_count += 1;
    }
    
    println!("Capture finished. Total height: {}", full_image.height());
    
    // Convert to Base64
    let base64_img = image_to_base64(&full_image).map_err(|e| e.to_string())?;
    
    // Emit event with result
    app.emit("capture-complete", base64_img).map_err(|e| e.to_string())?;
    
    Ok(())
}

fn capture_rect(x: i32, y: i32, width: u32, height: u32) -> Result<DynamicImage, String> {
    let screens = Screen::all().map_err(|e| format!("Failed to get screens: {}", e))?;
    
    // Find the screen that contains the point (x, y)
    let screen = screens.iter().find(|s| {
        let sx = s.display_info.x;
        let sy = s.display_info.y;
        let sw = s.display_info.width;
        let sh = s.display_info.height;
        
        let cx = x + (width as i32 / 2);
        let cy = y + (height as i32 / 2);
        
        cx >= sx && cx < sx + sw as i32 && cy >= sy && cy < sy + sh as i32
    }).or(screens.first()).ok_or("No screen found")?;

    let scale = screen.display_info.scale_factor;
    
    let rx_logical = x - screen.display_info.x;
    let ry_logical = y - screen.display_info.y;

    let rx = (rx_logical as f32 * scale) as i32;
    let ry = (ry_logical as f32 * scale) as i32;
    let rw = (width as f32 * scale) as u32;
    let rh = (height as f32 * scale) as u32;

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
