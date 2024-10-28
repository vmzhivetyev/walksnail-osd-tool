use std::time::Duration;

use backend::ffmpeg::{FromFfmpegMessage, VideoInfo};

#[derive(Default)]
pub struct RenderStatus {
    pub decoder_status: Status,
    pub encoder_status: Status,
}

#[derive(PartialEq, Default)]
pub enum Status {
    #[default]
    Idle,
    InProgress {
        time_remaining: Option<Duration>,
        fps: f32,
        speed: f32,
        bitrate_kbps: f32,
        progress_pct: f32,
    },
    Completed,
    Cancelled {
        progress_pct: f32,
    },
    Error {
        progress_pct: f32,
        error: String,
    },
}

impl RenderStatus {
    pub fn start_render(&mut self) {
        self.decoder_status = Status::InProgress {
            time_remaining: None,
            fps: 0.0,
            speed: 0.0,
            bitrate_kbps: 0.0,
            progress_pct: 0.0,
        };
    }

    pub fn stop_render(&mut self) {
        if let Status::InProgress { progress_pct, .. } = self.decoder_status {
            self.decoder_status = Status::Cancelled { progress_pct }
        }
    }

    pub fn reset(&mut self) {
        self.decoder_status = Status::Idle;
    }

    fn finished(&mut self) {
        self.decoder_status = Status::Completed;
    }

    fn error(&mut self, error: &str) {
        self.decoder_status = Status::Error {
            progress_pct: 0.0,
            error: error.into(),
        }
    }

    pub fn update_from_ffmpeg_message(&mut self, message: FromFfmpegMessage, video_info: &VideoInfo) {
        match (&self.decoder_status, &message) {
            (
                Status::InProgress { progress_pct, .. },
                FromFfmpegMessage::DecoderFatalError(e) | FromFfmpegMessage::EncoderFatalError(e),
            ) => {
                dbg!("🌤️🌤️🌤️", e);
                self.decoder_status = Status::Error {
                    progress_pct: *progress_pct,
                    error: e.clone(),
                }
            }

            (Status::InProgress { .. }, FromFfmpegMessage::DecoderProgress(p)) => {
                let frame = p.frame as f32;
                let total_frames = video_info.total_frames as f32;
                let progress_pct = frame / total_frames;
                let frames_remaining = total_frames - frame;
                let time_remaining_secs = frames_remaining / p.fps;
                // dbg!("🌤️🌤️🌤️", frame, total_frames, progress_pct, frames_remaining, time_remaining_secs);
                self.decoder_status = Status::InProgress {
                    time_remaining: if time_remaining_secs.is_finite() && time_remaining_secs.is_sign_positive() {
                        Some(Duration::from_secs_f32(time_remaining_secs))
                    } else {
                        None
                    },
                    fps: p.fps,
                    speed: p.speed,
                    bitrate_kbps: p.bitrate_kbps,
                    progress_pct,
                };
            }

            (Status::InProgress { .. }, FromFfmpegMessage::EncoderProgress(p)) => {
                let frame = p.frame as f32;
                let total_frames = video_info.total_frames as f32;
                let progress_pct = frame / total_frames;
                let frames_remaining = total_frames - frame;
                let time_remaining_secs = frames_remaining / p.fps;
                self.encoder_status = Status::InProgress {
                    time_remaining: if time_remaining_secs.is_finite() && time_remaining_secs.is_sign_positive() {
                        Some(Duration::from_secs_f32(time_remaining_secs))
                    } else {
                        None
                    },
                    fps: p.fps,
                    speed: p.speed,
                    bitrate_kbps: p.bitrate_kbps,
                    progress_pct,
                };
            }

            (Status::InProgress { .. }, FromFfmpegMessage::DecoderFinished) => self.finished(),

            // The decoder should always finish first so if the encoder finished when the render is in progress it must be an error.
            // In practice the encoder sometimes reaches EOF early so we only report an error when the encoder finishes
            // and progress, as reported by the decoder, is (near) zero.
            (Status::InProgress { progress_pct, .. }, FromFfmpegMessage::EncoderFinished) if *progress_pct < 0.001 => {
                self.error("Encoder unexpectedly finished")
            }

            _ => {}
        }
    }

    pub fn is_in_progress(&self) -> bool {
        matches!(self.decoder_status, Status::InProgress { .. })
    }

    pub fn is_not_in_progress(&self) -> bool {
        !self.is_in_progress()
    }
}
