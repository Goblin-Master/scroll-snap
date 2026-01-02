use image::{DynamicImage, GenericImage, GenericImageView, Rgba};

/// Calculate the overlap height between two images
/// prev_img: The previous screenshot (we look at the bottom of this)
/// curr_img: The new screenshot (we look at the top of this)
/// Returns: The Y-coordinate in `curr_img` where the content starts to *differ* from `prev_img` bottom.
///          Effectively, this is the height of the overlapping region in `curr_img`.
pub fn calculate_overlap(prev_img: &DynamicImage, curr_img: &DynamicImage) -> u32 {
    let width = prev_img.width();
    let prev_height = prev_img.height();
    let curr_height = curr_img.height();
    
    // We only scan the top 50% of the new image to find where the previous image ended.
    // If the user scrolled more than a screen height, we can't stitch anyway.
    let scan_depth = (curr_height / 2).min(prev_height / 2);

    // We use a large block for signature matching to avoid false positives with repeated patterns (like code lines).
    // Let's use the bottom 20% of the previous image, or at least 50 pixels.
    let signature_height = (prev_height / 5).max(50).min(prev_height);
    let signature_start_y = prev_height - signature_height;
    
    // Safety check
    if width == 0 || prev_height == 0 || curr_height == 0 {
        return 0;
    }

    // Optimization: Instead of checking every pixel, we check a grid.
    // If a candidate row matches, we do a full verify.
    
    // We search in `curr_img` for the start of the `signature` block.
    // If `prev_img` bottom matches `curr_img` at offset `y`, then `y` is the start of the match.
    // The overlap region in `curr_img` is from `0` to `y + signature_height`.
    // Wait, let's trace:
    // prev: [ ... A B C ] (C is bottom signature)
    // curr: [ B C D ... ]
    // We find C at `curr` offset `y`.
    // Then `curr` rows 0..y are "B". "B" is also in `prev`.
    // So the overlap is `curr` rows 0..(y + signature_height).
    // The new content starts at `y + signature_height`.
    
    // We iterate `y` representing the top-shift of the signature in the new image.
    for y in 0..scan_depth {
        // If the signature block would go out of bounds in curr_img, stop.
        if y + signature_height > curr_height {
            break;
        }
        
        // Fast check: Compare the first, middle, and last row of the signature block
        if check_row_match(prev_img, signature_start_y, curr_img, y, width) &&
           check_row_match(prev_img, signature_start_y + signature_height / 2, curr_img, y + signature_height / 2, width) &&
           check_row_match(prev_img, signature_start_y + signature_height - 1, curr_img, y + signature_height - 1, width) 
        {
            // Potential match found, do strict full block comparison
            if compare_blocks_strict(prev_img, signature_start_y, curr_img, y, width, signature_height) {
                println!("Stitch Match: Found overlap at y={}, overlap height={}", y, y + signature_height);
                return y + signature_height;
            }
        }
    }

    // No match found
    0
}

fn check_row_match(img1: &DynamicImage, y1: u32, img2: &DynamicImage, y2: u32, width: u32) -> bool {
    let step = 10; // Check every 10th pixel for speed
    let tolerance = 5; // Very strict tolerance
    
    for x in (0..width).step_by(step) {
        let p1 = img1.get_pixel(x, y1);
        let p2 = img2.get_pixel(x, y2);
        if !pixels_are_similar(p1, p2, tolerance) {
            return false;
        }
    }
    true
}

fn compare_blocks_strict(img1: &DynamicImage, y1: u32, img2: &DynamicImage, y2: u32, width: u32, height: u32) -> bool {
    let step = 2; // Check every 2nd pixel
    let tolerance = 10; // Strict tolerance
    let mut diff_count = 0;
    let max_diff = (width * height / step / step) / 100; // Allow max 1% different pixels (noise)
    
    for h in (0..height).step_by(step as usize) {
        for x in (0..width).step_by(step as usize) {
            let p1 = img1.get_pixel(x, y1 + h);
            let p2 = img2.get_pixel(x, y2 + h);
            
            if !pixels_are_similar(p1, p2, tolerance) {
                diff_count += 1;
                if diff_count > max_diff {
                    return false;
                }
            }
        }
    }
    true
}

fn pixels_are_similar(p1: Rgba<u8>, p2: Rgba<u8>, tolerance: i32) -> bool {
    let r_diff = (p1[0] as i32 - p2[0] as i32).abs();
    let g_diff = (p1[1] as i32 - p2[1] as i32).abs();
    let b_diff = (p1[2] as i32 - p2[2] as i32).abs();
    
    r_diff <= tolerance && g_diff <= tolerance && b_diff <= tolerance
}

pub fn append_image(base: &DynamicImage, new_part: &DynamicImage, overlap_height: u32) -> DynamicImage {
    let width = base.width();
    let base_height = base.height();
    let new_height = new_part.height();
    
    // We only keep the part of `new_part` that is NOT in the overlap.
    // overlap_height is the amount of pixels in `new_part` (from top) that duplicates `base`.
    // So we want `new_part` from `overlap_height` to end.
    
    // Safety check
    if overlap_height >= new_height {
        // The entire new image is a duplicate? Return base.
        return base.clone();
    }
    
    let append_height = new_height - overlap_height;
    let final_height = base_height + append_height;

    let mut final_img = DynamicImage::new_rgba8(width, final_height);
    
    // Copy base image
    let _ = final_img.copy_from(base, 0, 0);
    
    // Copy non-overlapping part of new image
    // Source rect: x=0, y=overlap_height, w=width, h=append_height
    // Dest: x=0, y=base_height
    let crop = new_part.view(0, overlap_height, width, append_height);
    // Convert SubImage to DynamicImage (ImageBuffer) to satisfy GenericImageView trait for copy_from
    let crop_img = crop.to_image();
    let _ = final_img.copy_from(&crop_img, 0, base_height);
    
    final_img
}
