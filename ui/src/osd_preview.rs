use backend::{
    font,
    osd::{self, OsdOptions},
    overlay::{overlay_dji_srt_data, overlay_osd, overlay_srt_data, overlay_srt_debug_data},
    srt::{self, SrtOptions},
    util::Dimension,
};
use image::RgbaImage;

#[tracing::instrument(skip(osd_frame, srt_file, srt_frame_index, font), level = "debug")]
pub fn create_osd_preview(
    width: u32,
    height: u32,
    osd_frame: &osd::Frame,
    srt_file: Option<&srt::SrtFile>,
    srt_frame_index: Option<usize>,
    font: &font::FontFile,
    srt_font: &rusttype::Font,
    osd_options: &OsdOptions,
    srt_options: &SrtOptions,
) -> RgbaImage {
    let mut image = RgbaImage::new(width, height);

    overlay_osd(&mut image, osd_frame, font, osd_options);

    if !srt_options.no_srt {
        if let (Some(srt_file), Some(frame_index)) = (srt_file, srt_frame_index) {
            if let Some(frame_data) = srt_file.data.get_frame_data(frame_index) {
                match frame_data {
                    backend::srt::SrtFrameDataRef::Normal(data) => {
                        overlay_srt_data(&mut image, data, srt_font, srt_options);
                    }
                    backend::srt::SrtFrameDataRef::Debug(data) => {
                        overlay_srt_debug_data(&mut image, data, srt_font, srt_options);
                    }
                    backend::srt::SrtFrameDataRef::Dji(data) => {
                        overlay_dji_srt_data(&mut image, data, srt_font, srt_options);
                    }
                }
            }
        }
    }

    image
}

#[tracing::instrument(level = "debug")]
pub fn calculate_horizontal_offset(width: u32, osd_frame: &osd::Frame, character_bounds_size: Dimension<u32>) -> i32 {
    let min_x_grid = osd_frame.glyphs.iter().map(|g| g.grid_position.x).min().unwrap();
    let max_x_grid = osd_frame.glyphs.iter().map(|g| g.grid_position.x).max().unwrap();
    let pixel_range = (max_x_grid - min_x_grid + 1) * character_bounds_size.width;
    let offset = (width - pixel_range) / 2 - min_x_grid * character_bounds_size.width;
    offset as i32
}

#[tracing::instrument(level = "debug")]
pub fn calculate_vertical_offset(height: u32, osd_frame: &osd::Frame, character_bounds_size: Dimension<u32>) -> i32 {
    let min_y_grid = osd_frame.glyphs.iter().map(|g| g.grid_position.y).min().unwrap();
    let max_y_grid = osd_frame.glyphs.iter().map(|g| g.grid_position.y).max().unwrap();
    let pixel_range = (max_y_grid - min_y_grid + 1) * character_bounds_size.height;
    let offset = (height - pixel_range) / 2 - min_y_grid * character_bounds_size.height;
    offset as i32
}
