use std::time::Instant;

use image::{imageops::overlay, RgbaImage};

use crate::{
    font::{self},
    osd::{self, OsdOptions},
    util::Dimension,
};

/// Fast overlay for images that are typically either fully opaque or fully transparent
/// (like OSD glyphs). Falls back to standard overlay for complex alpha.
fn fast_overlay(bottom: &mut RgbaImage, top: &RgbaImage, x: i64, y: i64) {
    let bottom_dims = bottom.dimensions();
    let top_dims = top.dimensions();

    // Check if overlay is completely outside bounds
    if x >= bottom_dims.0 as i64 || y >= bottom_dims.1 as i64 ||
       x + top_dims.0 as i64 <= 0 || y + top_dims.1 as i64 <= 0 {
        return;
    }

    // Check if this is a simple case (fully opaque or fully transparent pixels)
    let mut has_complex_alpha = false;
    for pixel in top.pixels() {
        let alpha = pixel[3];
        if alpha != 0 && alpha != 255 {
            has_complex_alpha = true;
            break;
        }
    }

    if !has_complex_alpha {
        // Fast path: copy pixels directly (no alpha blending needed)
        let start_x = x.max(0) as u32;
        let start_y = y.max(0) as u32;
        let end_x = (x + top_dims.0 as i64).min(bottom_dims.0 as i64) as u32;
        let end_y = (y + top_dims.1 as i64).min(bottom_dims.1 as i64) as u32;

        let offset_x = if x < 0 { -x as u32 } else { 0 };
        let offset_y = if y < 0 { -y as u32 } else { 0 };

        for ty in offset_y..(end_y - start_y + offset_y) {
            for tx in offset_x..(end_x - start_x + offset_x) {
                let top_pixel = top.get_pixel(tx, ty);
                if top_pixel[3] == 255 { // Fully opaque
                    bottom.put_pixel(start_x + tx - offset_x, start_y + ty - offset_y, *top_pixel);
                }
                // Skip fully transparent pixels (alpha = 0)
            }
        }
    } else {
        // Fallback to standard overlay for complex alpha
        overlay(bottom, top, x, y);
    }
}

pub fn get_ideal_character_size(frame_width: u32, frame_height: u32) -> Dimension<u32> {
    let char_height = frame_height / osd::OSD_GRID_HEIGHT;
    let char_width = frame_width / osd::OSD_GRID_WIDTH;
    Dimension {
        width: char_width,
        height: char_height,
    }
}

#[inline]
pub fn overlay_osd(image: &mut RgbaImage, osd_frame: &osd::Frame, font: &font::FontFile, osd_options: &OsdOptions) {
    // TODO: check if this can be run in parallel
    let character_size_class = osd_options
        .character_size_class
        .clone()
        .unwrap_or(font::CharacterSizeClass::Normal);

    let char_desired_size = get_ideal_character_size(image.width(), image.height());

    let _start = Instant::now();
    let mut _rendered_chars = 0;

    for character in &osd_frame.glyphs {
        if character.index == 0 || osd_options.get_mask(&character.grid_position) {
            continue;
        }
        
        if let Some(character_image) = font.get_character(
            character.index as usize,
            &character_size_class,
            char_desired_size.to_owned(),
        ) {
            let grid_position = &character.grid_position;

            // According to https://betaflight.com/docs/wiki/configurator/osd-tab
            // INFO
            // HD OSD defaults to a 53 column x 20 row grid of OSD elements.
            // When the VTX is online BetaFlight will query via MSP Displayport to determine the optimum grid size and may update the grid to match what is supported by the digital VTX system
            const ROW_COUNT: u32 = 20;
            const COL_COUNT: u32 = 53;

            // Important: integer division here.
            let single_glyph_x_offset = image.width() / COL_COUNT;
            let single_glyph_y_offset = image.height() / ROW_COUNT;
            let remainder_x_offset = image.width() % COL_COUNT / 2;
            let remainder_y_offset = image.height() % ROW_COUNT / 2;

            let x_raw = remainder_x_offset + grid_position.x * single_glyph_x_offset;
            let y_raw = remainder_y_offset + grid_position.y * single_glyph_y_offset;
            let x = (x_raw as i32 + osd_options.position.x) as i64;
            let y = (y_raw as i32 + osd_options.position.y) as i64;

            _rendered_chars += 1;

            fast_overlay(image, &character_image, x, y)
        }
    }

    // tracing::info!(
    //     "overlay_osd done in {:?} for {} chars.",
    //     _start.elapsed(),
    //     _rendered_chars
    // );
}
