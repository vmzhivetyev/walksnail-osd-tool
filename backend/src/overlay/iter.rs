use std::{iter::Peekable, vec::IntoIter};

use crossbeam_channel::{Receiver, Sender};
use ffmpeg_sidecar::{
    child::FfmpegChild,
    event::{FfmpegEvent, OutputVideoFrame},
    iter::FfmpegIterator,
};
use image::{Rgba, RgbaImage};

use super::{overlay_osd, overlay_srt_data, overlay_srt_debug_data};
use crate::{
    ffmpeg::{handle_decoder_events, FromFfmpegMessage, ToFfmpegMessage},
    font,
    osd::{self, OsdOptions},
    srt::{self, SrtFrame, SrtOptions},
};

pub struct FrameOverlayIter<'a> {
    decoder_iter: FfmpegIterator,
    decoder_process: FfmpegChild,
    osd_frames_iter: Peekable<IntoIter<osd::Frame>>,
    srt_frames_iter: Peekable<IntoIter<srt::SrtFrame>>,
    font_file: font::FontFile,
    osd_options: OsdOptions,
    srt_options: SrtOptions,
    srt_font: rusttype::Font<'a>,
    current_osd_frame: osd::Frame,
    current_srt_frame: Option<srt::SrtFrame>,
    ffmpeg_sender: Sender<FromFfmpegMessage>,
    ffmpeg_receiver: Receiver<ToFfmpegMessage>,
    chroma_key: Option<Rgba<u8>>,
}

impl<'a> FrameOverlayIter<'a> {
    #[tracing::instrument(skip(decoder_iter, decoder_process, osd_frames, font_file), level = "debug")]
    pub fn new(
        decoder_iter: FfmpegIterator,
        decoder_process: FfmpegChild,
        osd_frames: Vec<osd::Frame>,
        srt_frames: Option<Vec<srt::SrtFrame>>,
        font_file: font::FontFile,
        srt_font: rusttype::Font<'a>,
        osd_options: &OsdOptions,
        srt_options: &SrtOptions,
        ffmpeg_sender: Sender<FromFfmpegMessage>,
        ffmpeg_receiver: Receiver<ToFfmpegMessage>,
        chroma_key: Option<[f32; 4]>,
    ) -> Self {
        let mut osd_frames_iter = osd_frames.into_iter();

        let mut srt_frames_iter = srt_frames
            .map(|frames| frames.into_iter().peekable())
            .unwrap_or_else(|| Vec::<SrtFrame>::new().into_iter().peekable());

        let first_osd_frame = if osd_options.osd_playback_offset >= 0.0 {
            osd::Frame::default()
        } else {
            osd_frames_iter.next().unwrap()
        };

        let first_srt_frame = srt_frames_iter.next();

        let chroma_key = chroma_key.map(|c| {
            Rgba([
                (c[0] * 255.0) as u8,
                (c[1] * 255.0) as u8,
                (c[2] * 255.0) as u8,
                (c[3] * 255.0) as u8,
            ])
        });
        Self {
            decoder_iter,
            decoder_process,
            osd_frames_iter: osd_frames_iter.peekable(),
            srt_frames_iter: srt_frames_iter,
            font_file,
            osd_options: osd_options.clone(),
            srt_options: srt_options.clone(),
            srt_font: srt_font.clone(),
            current_osd_frame: first_osd_frame,
            current_srt_frame: first_srt_frame,
            ffmpeg_sender,
            ffmpeg_receiver,
            chroma_key,
        }
    }
}

impl Iterator for FrameOverlayIter<'_> {
    type Item = OutputVideoFrame;

    fn next(&mut self) -> Option<Self::Item> {
        //  On every iteration check if the render should be stopped
        while let Ok(ToFfmpegMessage::AbortRender) = self.ffmpeg_receiver.try_recv() {
            self.decoder_process.quit().unwrap();
        }

        self.decoder_iter.find_map(|e| match e {
            FfmpegEvent::OutputFrame(mut video_frame) => {
                let _start = std::time::Instant::now();

                // For every video frame check if frame time is later than the next OSD frame time.
                // If so advance the iterator over the OSD frames so we use the correct OSD frame
                // for this video frame
                if let Some(next_osd_frame) = self.osd_frames_iter.peek() {
                    let next_osd_frame_secs =
                        self.osd_options.osd_playback_offset + (next_osd_frame.time_millis as f32 / 1000.0);
                    if video_frame.timestamp > next_osd_frame_secs * self.osd_options.osd_playback_speed_factor {
                        self.current_osd_frame = self.osd_frames_iter.next().unwrap();
                    }
                }

                if let Some(next_srt_frame) = self.srt_frames_iter.peek() {
                    let next_srt_start_time_secs = next_srt_frame.start_time_secs;
                    if video_frame.timestamp > next_srt_start_time_secs {
                        self.current_srt_frame = self.srt_frames_iter.next();
                    }
                }

                let mut frame_image = if let Some(chroma_key) = self.chroma_key {
                    // this should support alpha
                    RgbaImage::from_pixel(video_frame.width, video_frame.height, chroma_key)
                } else {
                    RgbaImage::from_raw(video_frame.width, video_frame.height, video_frame.data).unwrap()
                };

                if !self.osd_options.no_osd {
                    overlay_osd(
                        &mut frame_image,
                        &self.current_osd_frame,
                        &self.font_file,
                        &self.osd_options,
                    );
                }

                if !self.srt_options.no_srt {
                    if let Some(frame) = &self.current_srt_frame {
                        if let Some(srt_data) = &frame.data {
                            overlay_srt_data(&mut frame_image, srt_data, &self.srt_font, &self.srt_options);
                        }

                        if let Some(srt_debug_data) = &frame.debug_data {
                            overlay_srt_debug_data(&mut frame_image, srt_debug_data, &self.srt_font, &self.srt_options);
                        }
                    }
                }

                video_frame.data = frame_image.into_raw();

                // tracing::info!(
                //     "next frame prepared in {:?}.",
                //     _start.elapsed()
                // );

                Some(video_frame)
            }
            other_event => {
                let _start = std::time::Instant::now();

                tracing::trace!("{:?}", &other_event);
                // dbg!("🌤️🌤️🌤️", &other_event);
                handle_decoder_events(other_event, &self.ffmpeg_sender);

                // tracing::info!(
                //     "handle_decoder_events done in {:?}.",
                //     _start.elapsed()
                // );

                None
            }
        })
    }
}
