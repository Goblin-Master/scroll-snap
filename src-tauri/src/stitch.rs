use image::{DynamicImage, GenericImageView, Rgba, ImageBuffer, RgbaImage};

/// Calculate the overlap height between two images
/// prev_img: The previous screenshot
/// curr_img: The new screenshot after scrolling
pub fn calculate_overlap(prev_img: &DynamicImage, curr_img: &DynamicImage) -> u32 {
    let width = prev_img.width();
    let prev_height = prev_img.height();
    
    // We assume each scroll won't exceed 1/2 of screen height to reduce calculation
    // Start from top of curr_img, look for match in bottom of prev_img
    let scan_depth = prev_height / 2; 

    // Take the last row of prev_img as the feature row
    let target_row_y = prev_height - 1;
    
    // Safety check
    if width == 0 || prev_height == 0 || curr_img.height() == 0 {
        return 0;
    }

    let target_row_pixels: Vec<Rgba<u8>> = (0..width)
        .map(|x| prev_img.get_pixel(x, target_row_y))
        .collect();

    // Search for this row in the first scan_depth rows of curr_img
    for y in 0..scan_depth {
        if y >= curr_img.height() {
            break;
        }
        
        let mut match_found = true;
        // Sample comparison: compare every 5th pixel for performance
        for x in (0..width).step_by(5) {
            let curr_pixel = curr_img.get_pixel(x, y);
            let target_pixel = target_row_pixels[x as usize];
            
            // Allow small color difference (tolerance)
            if !pixels_are_similar(curr_pixel, target_pixel, 10) { // Increased tolerance slightly
                match_found = false;
                break;
            }
        }

        if match_found {
            // Found it! curr_img row y corresponds to prev_img last row.
            // The overlap part is curr_img 0..=y
            // So the overlap height is y + 1 (since 0-indexed)
            // Wait, SRS says: "Overlap part is curr_img 0 to y (not including y)" -> "return y"
            // Let's trace:
            // If prev_img last row matches curr_img row 0. Overlap is 1 pixel?
            // If y=0 matches, it means row 0 of curr is row LAST of prev.
            // Then we discard row 0 of curr?
            // If we assume the scroll moves content UP, new content appears at bottom.
            // Wait, "Scroll Down" means content moves UP.
            // So prev_img bottom should match curr_img top.
            // If prev_img last row matches curr_img row y.
            // Then curr_img rows 0..y are the same as prev_img rows (H-1-y)..(H-1).
            // So overlap height is y + 1.
            // SRS says "return y". "Overlap part is curr_img 0 to y (not including y)".
            // If y=0 matches, overlap is 0? That implies row 0 is SAME as last row.
            // If row 0 is same as last row, then we have 1 line overlap.
            // I will return y + 1 as overlap height.
            // Or maybe SRS logic is: y is the index in curr_img that matches last row of prev_img.
            // So curr_img[0..=y] is the overlap.
            // We want to append curr_img[y+1..].
            // So we return y (index).
            return y; 
        }
    }

    0 // No overlap found
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
