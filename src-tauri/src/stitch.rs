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
    // Take 10 rows from the bottom of prev_img for better signature
    let signature_height = 10.min(prev_height);
    let signature_start_y = prev_height - signature_height;
    
    // Search for this signature in the top part of curr_img
    for y in 0..scan_depth {
        if y + signature_height > curr_height {
            break;
        }
        
        // Compare the signature block
        if compare_blocks(prev_img, signature_start_y, curr_img, y, width, signature_height) {
            // Found overlap!
            return y + signature_height - 1;
        }
    }

    0 // No overlap found
}

fn compare_blocks(img1: &DynamicImage, y1: u32, img2: &DynamicImage, y2: u32, width: u32, height: u32) -> bool {
    let step = 2; // Check every 2nd pixel for higher accuracy
    let tolerance = 20; // Increased tolerance slightly for anti-aliasing differences
    
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
