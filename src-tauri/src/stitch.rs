use image::{DynamicImage, GenericImageView, Rgba, ImageBuffer, RgbaImage};

/// Calculate the overlap height between two images
/// prev_img: The previous screenshot
/// curr_img: The new screenshot after scrolling
pub fn calculate_overlap(prev_img: &DynamicImage, curr_img: &DynamicImage) -> u32 {
    let width = prev_img.width();
    let prev_height = prev_img.height();
    let curr_height = curr_img.height();
    
    // We assume each scroll won't exceed 1/2 of screen height to reduce calculation
    let scan_depth = prev_height / 2; 

    // Safety check
    if width == 0 || prev_height == 0 || curr_height == 0 {
        return 0;
    }

    // Use a multi-row signature for robustness
    // Take 5 rows from the bottom of prev_img (skipping the very last pixel row to avoid border issues)
    let signature_height = 5.min(prev_height);
    let signature_start_y = prev_height - signature_height;
    
    // Search for this signature in the top part of curr_img
    for y in 0..scan_depth {
        if y + signature_height > curr_height {
            break;
        }
        
        // Compare the signature block
        if compare_blocks(prev_img, signature_start_y, curr_img, y, width, signature_height) {
            // Match found! 
            // The signature at prev_img[prev_height - sig_h] matches curr_img[y]
            // This means the overlap is at curr_img[y + sig_h] (roughly)
            // Wait, logic check:
            // prev_img: [ ... content ... signature ]
            // curr_img: [ ... content ... signature ... new ]
            // NO, scroll down means content moves UP.
            // prev_img: [ A B C ]
            // curr_img: [ B C D ]
            // We look for "C" (bottom of prev) in curr.
            // "C" should be at the TOP of curr (or near top).
            // So if prev_img bottom matches curr_img row `y`.
            // Then `y` is the start of the overlap in curr_img.
            // And the overlap ends at `y + signature_height`? No.
            // The overlap is everything from 0 to `y + signature_height` in curr_img that matches prev_img.
            // Actually, if prev_img's bottom matches curr_img's row `y`, 
            // it means curr_img[0..y] is OLD content (that was above the bottom of prev_img),
            // and curr_img[y..] is the content starting from where prev_img ended?
            // Let's draw:
            // Prev:
            // Line 100: Hello
            // Line 101: World (Bottom)
            //
            // Curr (Scrolled down 1 line):
            // Line 0: World
            // Line 1: !
            //
            // We search for "World" (Prev bottom) in Curr.
            // We find "World" at Curr Row 0.
            // So overlap index is 0?
            // If overlap index is 0, we append from 1. Correct.
            //
            // Curr (Scrolled down 50 lines):
            // Line 0..49: (Content that was in Prev)
            // Line 50: World
            //
            // We find "World" at Row 50.
            // So we append from 51.
            // So overlap_index (the matching row) is `y` (plus signature offset?).
            // If signature is 5 rows.
            // Prev: rows 95-99 are "Signature".
            // Curr: rows 50-54 are "Signature".
            // So Prev[95] == Curr[50].
            // This means Curr has shifted up by 95-50 = 45 pixels? No.
            // It means Curr[50] corresponds to Prev[95].
            // So Curr[0..50] corresponds to Prev[45..95].
            // And Curr[54] is Prev[99].
            // So everything in Curr up to 54 is in Prev.
            // So we should append starting from 55.
            // So overlap height is y + signature_height.
            // And we return index = y + signature_height - 1.
            
            return y + signature_height - 1;
        }
    }

    0 // No overlap found
}

fn compare_blocks(img1: &DynamicImage, y1: u32, img2: &DynamicImage, y2: u32, width: u32, height: u32) -> bool {
    let step = 5; // Check every 5th pixel for speed
    let tolerance = 15; // Color tolerance
    
    for h in 0..height {
        for x in (0..width).step_by(step) {
            let p1 = img1.get_pixel(x, y1 + h);
            let p2 = img2.get_pixel(x, y2 + h);
            
            if !pixels_are_similar(p1, p2, tolerance) {
                return false;
            }
        }
    }
    true
}

fn pixels_are_similar(p1: Rgba<u8>, p2: Rgba<u8>, tolerance: i32) -> bool {
    let r_diff = (p1[0] as i32 - p2[0] as i32).abs();
    let g_diff = (p1[1] as i32 - p2[1] as i32).abs();
    let b_diff = (p1[2] as i32 - p2[2] as i32).abs();
    (r_diff + g_diff + b_diff) <= tolerance
}

/// Append new_img to base_img, skipping the first `overlap` rows of new_img
pub fn append_image(base_img: &DynamicImage, new_img: &DynamicImage, overlap_index: u32) -> DynamicImage {
    let base_width = base_img.width();
    let base_height = base_img.height();
    let new_width = new_img.width();
    let new_height = new_img.height();

    // The part of new_img to append starts from overlap_index + 1
    // If overlap_index is the row that matched the last row of base_img.
    // Then we skip 0..=overlap_index.
    // So start_y = overlap_index + 1.
    let start_y = overlap_index + 1;
    
    if start_y >= new_height {
        return base_img.clone();
    }

    let append_height = new_height - start_y;
    let final_width = base_width.max(new_width);
    let final_height = base_height + append_height;

    let mut final_img: RgbaImage = ImageBuffer::new(final_width, final_height);

    // Copy base image
    // copy_from is available on GenericImage, but for DynamicImage we might need to be careful
    // We can iterate or use sub_image (which might be slow).
    // Let's copy pixel by pixel or use `copy_from` if compatible.
    // DynamicImage implements GenericImage.
    
    // Copy base
    for y in 0..base_height {
        for x in 0..base_width {
            final_img.put_pixel(x, y, base_img.get_pixel(x, y));
        }
    }

    // Copy new image (cropped)
    for y in 0..append_height {
        for x in 0..new_width {
            let src_y = start_y + y;
            final_img.put_pixel(x, base_height + y, new_img.get_pixel(x, src_y));
        }
    }

    DynamicImage::ImageRgba8(final_img)
}
