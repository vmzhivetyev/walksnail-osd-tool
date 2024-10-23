use image::{imageops::overlay, RgbaImage};
use std::time::Instant;

use crate::{
    font::{self, CharacterSize},
    osd::{self, OsdOptions},
};

pub fn get_character_size(lines: u32) -> CharacterSize {
    match lines {
        540 => CharacterSize::Race,
        720 => CharacterSize::Small,
        1080 => CharacterSize::Large,
        1440 => CharacterSize::XLarge,
        2160 => CharacterSize::Ultra,
        _ => CharacterSize::Large,
    }
}

#[inline]
pub fn overlay_osd(image: &mut RgbaImage, osd_frame: &osd::Frame, font: &font::FontFile, osd_options: &OsdOptions) {
    // TODO: check if this can be run in parallel
    let osd_character_size = osd_options
        .character_size
        .clone()
        .unwrap_or(get_character_size(image.height()));

    let _start = Instant::now();
    let mut _rendered_chars = 0;

    for character in &osd_frame.glyphs {
        if character.index == 0 || osd_options.get_mask(&character.grid_position) {
            continue;
        }
        if let Some(character_image) = font.get_character(character.index as usize, &osd_character_size) {
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

            overlay(image, &character_image, x, y)
        }
    }

    // tracing::info!(
    //     "overlay_osd done in {:?} for {} chars.",
    //     _start.elapsed(),
    //     _rendered_chars
    // );
}
