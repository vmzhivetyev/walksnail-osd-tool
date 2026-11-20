use std::{iter::Peekable, sync::Arc, vec::IntoIter};

use image::{Rgba, RgbaImage};

use super::{overlay_osd, overlay_srt_data, overlay_srt_debug_data};
use crate::{
    font,
    osd::{self, OsdOptions},
    srt::{self, SrtFrame, SrtOptions},
};

/// Generates the overlay layer (OSD and SRT) at a fixed frame rate, without touching the video itself.
/// ffmpeg composites these frames onto the decoded video.
///
/// OSD and SRT data change far less often than the video frame rate, so when nothing changed since the
/// previous frame the same shared image is returned again instead of being redrawn.
pub struct OverlayFrameIter<'a> {
    frame_rate: f32,
    frame_count: u32,
    next_frame_index: u32,
    width: u32,
    height: u32,
    background: Rgba<u8>,
    osd_frames_iter: Peekable<IntoIter<osd::Frame>>,
    srt_frames_iter: Peekable<IntoIter<SrtFrame>>,
    font_file: font::FontFile,
    osd_options: OsdOptions,
    srt_options: SrtOptions,
    srt_font: rusttype::Font<'a>,
    current_osd_frame: osd::Frame,
    current_srt_frame: Option<SrtFrame>,
    last_image: Option<Arc<RgbaImage>>,
}

impl<'a> OverlayFrameIter<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        frame_rate: f32,
        frame_count: u32,
        width: u32,
        height: u32,
        osd_frames: Vec<osd::Frame>,
        srt_frames: Option<Vec<srt::SrtFrame>>,
        font_file: font::FontFile,
        srt_font: rusttype::Font<'a>,
        osd_options: &OsdOptions,
        srt_options: &SrtOptions,
        background: Option<[f32; 4]>,
    ) -> Self {
        let mut osd_frames_iter = osd_frames.into_iter().peekable();
        let mut srt_frames_iter = srt_frames.unwrap_or_default().into_iter().peekable();

        let first_osd_frame = if osd_options.osd_playback_offset >= 0.0 {
            osd::Frame::default()
        } else {
            osd_frames_iter.next().unwrap_or_default()
        };
        let first_srt_frame = srt_frames_iter.next();

        let background = background
            .map(|c| {
                Rgba([
                    (c[0] * 255.0) as u8,
                    (c[1] * 255.0) as u8,
                    (c[2] * 255.0) as u8,
                    (c[3] * 255.0) as u8,
                ])
            })
            .unwrap_or(Rgba([0, 0, 0, 0]));

        Self {
            frame_rate,
            frame_count,
            next_frame_index: 0,
            width,
            height,
            background,
            osd_frames_iter,
            srt_frames_iter,
            font_file,
            osd_options: osd_options.clone(),
            srt_options: srt_options.clone(),
            srt_font,
            current_osd_frame: first_osd_frame,
            current_srt_frame: first_srt_frame,
            last_image: None,
        }
    }

    /// Advance the OSD and SRT iterators up to `timestamp`. Returns true if either current frame changed.
    fn advance_to(&mut self, timestamp: f32) -> bool {
        let mut changed = false;

        while let Some(next_osd_frame) = self.osd_frames_iter.peek() {
            let next_osd_frame_secs =
                self.osd_options.osd_playback_offset + (next_osd_frame.time_millis as f32 / 1000.0);
            if timestamp > next_osd_frame_secs * self.osd_options.osd_playback_speed_factor {
                self.current_osd_frame = self.osd_frames_iter.next().unwrap();
                changed = true;
            } else {
                break;
            }
        }

        while let Some(next_srt_frame) = self.srt_frames_iter.peek() {
            if timestamp > next_srt_frame.start_time_secs {
                self.current_srt_frame = self.srt_frames_iter.next();
                changed = true;
            } else {
                break;
            }
        }

        changed
    }

    fn draw(&self) -> RgbaImage {
        let mut image = RgbaImage::from_pixel(self.width, self.height, self.background);

        if !self.osd_options.no_osd {
            overlay_osd(&mut image, &self.current_osd_frame, &self.font_file, &self.osd_options);
        }

        if !self.srt_options.no_srt {
            if let Some(frame) = &self.current_srt_frame {
                if let Some(srt_data) = &frame.data {
                    overlay_srt_data(&mut image, srt_data, &self.srt_font, &self.srt_options);
                }
                if let Some(srt_debug_data) = &frame.debug_data {
                    overlay_srt_debug_data(&mut image, srt_debug_data, &self.srt_font, &self.srt_options);
                }
            }
        }

        image
    }
}

impl Iterator for OverlayFrameIter<'_> {
    type Item = Arc<RgbaImage>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_frame_index >= self.frame_count {
            return None;
        }

        let timestamp = self.next_frame_index as f32 / self.frame_rate;
        self.next_frame_index += 1;

        let changed = self.advance_to(timestamp);

        if let (false, Some(last_image)) = (changed, &self.last_image) {
            return Some(last_image.clone());
        }

        let image = Arc::new(self.draw());
        self.last_image = Some(image.clone());
        Some(image)
    }
}
