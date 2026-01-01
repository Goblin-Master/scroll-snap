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
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

lazy_static! {
    static ref CAPTURE_STATES: Mutex<HashMap<String, Arc<Mutex<bool>>>> = Mutex::new(HashMap::new());
}

#[tauri::command]
pub async fn start_scroll_capture(app: AppHandle, x: i32, y: i32, width: u32, height: u32) -> Result<(), String> {
    println!("Starting manual scroll capture task at ({}, {}) {}x{}", x, y, width, height);
    
    // Force hide ALL windows to ensure input is not blocked
    // Iterate over all windows and hide them
    let windows = app.webview_windows();
    for (label, window) in windows {
        println!("Hiding window: {}", label);
        let _ = window.hide();
    }
    
    // Give the window manager some time to actually hide the window and release focus
    // This is crucial for input passthrough to work immediately
    thread::sleep(Duration::from_millis(500));

    // Create a stop flag for this capture session
    let stop_flag = Arc::new(Mutex::new(false));
    let stop_flag_clone = stop_flag.clone();
    
    // Store it so we can access it from stop command
    // We use a simple key "current" since we only allow one capture at a time
    CAPTURE_STATES.lock().unwrap().insert("current".to_string(), stop_flag);
    
    // Register global shortcut F9 to stop capture
    let stop_flag_shortcut = stop_flag_clone.clone();
    let app_handle = app.clone();
    
    // We use a shortcut string representation. 
    // Note: F9 is a good choice. 
    let shortcut_str = "F9";
    
    // Register the shortcut
    if let Err(e) = app.global_shortcut().register(shortcut_str) {
        println!("Failed to register shortcut: {}", e);
    }
    
    // We can't directly pass a closure to global_shortcut, we need to listen globally?
    // Tauri v2 global shortcut plugin works by registering and then we can listen to events?
    // Actually, `tauri-plugin-global-shortcut` doesn't support closures per shortcut easily in this context
    // without setup in `lib.rs`.
    // BUT, we can just use `app.listen_global` if the plugin emits events? No.
    //
    // Easier way: We spin up a thread that checks the shortcut state? No, that's polling.
    //
    // Better way: Since we are in `capture_loop`, we can check the state of the shortcut if possible?
    // Or we rely on the main event loop in `lib.rs` to handle it?
    //
    // Let's use a simpler approach for this ad-hoc task:
    // We will Register it here.
    // And in `lib.rs`, we should have set up the handler.
    // But we are editing `capture.rs` now.
    //
    // Wait, the robust way is to use `GlobalShortcutExt` on `app` to register, 
    // but the EVENT handling needs to be hooked up.
    //
    // Alternative: We can use `rdev` (Rust device events) to listen for F9 globally in our capture loop!
    // This is actually cleaner for a "blocking loop" scenario like ours because we don't need to mess with Tauri's main loop.
    // AND `rdev` works cross-platform for simple key detection.
    //
    // BUT `rdev` might require extra system deps.
    //
    // Let's stick to Tauri's plugin but we need to handle the event.
    // The plugin exposes `app.global_shortcut().on_shortcut(...)`? No.
    //
    // Let's look at how we can stop.
    // We can spawn a separate task that waits for the shortcut event?
    // 
    // Actually, let's just use `rdev` for the "Press F9 to stop" feature inside the loop?
    // No, `rdev` is blocking.
    //
    // Let's go back to `lib.rs` later to add the handler.
    // For now, in `capture.rs`, we just need to make sure the loop checks the flag.
    // AND we need to make sure `lib.rs` can flip that flag.
    // But `CAPTURE_STATES` is private to `capture.rs`.
    //
    // So we need to expose a public function `trigger_stop()` in `capture.rs` that `lib.rs` can call.
    // We already have `stop_scroll_capture` command.
    // So if `lib.rs` detects F9, it just calls `capture::stop_scroll_capture()`.
    // Perfect.
    
    // So here we just Register F9? 
    // If we register F9 here, we need to unregister it when done.
    
    let app_clone_for_cleanup = app.clone();
    
    // Spawn a thread to handle the long-running capture process
    std::thread::spawn(move || {
        let result = run_capture_loop(&app, x, y, width, height, stop_flag_clone);
        
        // Unregister shortcut when done
        let _ = app_clone_for_cleanup.global_shortcut().unregister(shortcut_str);
        
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
    let mut full_image = capture_rect(x, y, width, height).map_err(|e| e.to_string())?;
    
    let mut static_count = 0;
    let max_static_count = 30; // 3 seconds (30 * 100ms)
    
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
            // Stop if static for 3 seconds
            if static_count >= max_static_count {
                 println!("Static content detected for 3s. Auto-stopping.");
                 break;
            }
            continue;
        }
        
        // Check for no overlap (too fast or error)
        if overlap_index == 0 {
             static_count += 1;
             if static_count >= max_static_count {
                 println!("No overlap detected for 3s. Auto-stopping.");
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
    
    // Show ALL windows before emitting event
    let windows = app.webview_windows();
    for (label, window) in windows {
        println!("Showing window: {}", label);
        let _ = window.show();
        let _ = window.set_focus();
    }

    // Emit event with result
    app.emit("capture-complete", base64_img).map_err(|e| e.to_string())?;
    
    Ok(())
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
