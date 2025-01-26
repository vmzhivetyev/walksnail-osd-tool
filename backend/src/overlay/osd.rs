use image::{imageops::overlay, RgbaImage};
use std::time::Instant;

use crate::{
    ffmpeg, font::{self}, osd::{self, OsdOptions}, util::Dimension
};

pub fn get_ideal_character_size(frame_width: u32, frame_height: u32) -> Dimension<u32> {
    let char_height = frame_height / osd::OSD_GRID_HEIGHT;
    let char_width = frame_width / osd::OSD_GRID_WIDTH;
    Dimension { width: char_width, height: char_height }
}

#[inline]
pub fn overlay_osd(image: &mut RgbaImage, osd_frame: &osd::Frame, font: &font::FontFile, osd_options: &OsdOptions) {
    // TODO: check if this can be run in parallel
    let character_size_class = osd_options
        .character_size_class.clone()
        .unwrap_or(font::CharacterSizeClass::Normal);

    let char_desired_size = get_ideal_character_size(image.width(), image.height());

    let _start = Instant::now();
    let mut _rendered_chars = 0;

    for character in &osd_frame.glyphs {
        if character.index == 0 || osd_options.get_mask(&character.grid_position) {
            continue;
        }
        if let Some(character_image) = font.get_character(character.index as usize, &character_size_class, char_desired_size.to_owned()) {
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
