use std::time::Duration;

use backend::ffmpeg::{FromFfmpegMessage, VideoInfo};

#[derive(Default)]
pub struct RenderStatus {
    pub status: Status,
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
        self.status = Status::InProgress {
            time_remaining: None,
            fps: 0.0,
            speed: 0.0,
            bitrate_kbps: 0.0,
            progress_pct: 0.0,
        };
    }

    pub fn stop_render(&mut self) {
        if let Status::InProgress { progress_pct, .. } = self.status {
            self.status = Status::Cancelled { progress_pct }
        }
    }

    pub fn reset(&mut self) {
        self.status = Status::Idle;
    }

    pub fn update_from_ffmpeg_message(&mut self, message: FromFfmpegMessage, video_info: &VideoInfo) {
        let Status::InProgress { progress_pct, .. } = self.status else {
            return;
        };

        match message {
            FromFfmpegMessage::EncoderFatalError(e) => self.status = Status::Error { progress_pct, error: e },

            FromFfmpegMessage::EncoderProgress(p) => {
                let frame = p.frame as f32;
                let total_frames = video_info.total_frames as f32;
                let frames_remaining = total_frames - frame;
                let time_remaining_secs = frames_remaining / p.fps;
                self.status = Status::InProgress {
                    time_remaining: if time_remaining_secs.is_finite() && time_remaining_secs.is_sign_positive() {
                        Some(Duration::from_secs_f32(time_remaining_secs))
                    } else {
                        None
                    },
                    fps: p.fps,
                    speed: p.speed,
                    bitrate_kbps: p.bitrate_kbps,
                    progress_pct: (frame / total_frames).min(1.0),
                };
            }

            // The encoder produces the output video, so it finishing means the render is done,
            // unless it stopped before producing anything.
            FromFfmpegMessage::EncoderFinished if progress_pct < 0.001 => {
                self.status = Status::Error {
                    progress_pct,
                    error: "Encoder unexpectedly finished".into(),
                }
            }
            FromFfmpegMessage::EncoderFinished => self.status = Status::Completed,
        }
    }

    pub fn is_in_progress(&self) -> bool {
        matches!(self.status, Status::InProgress { .. })
    }

    pub fn is_not_in_progress(&self) -> bool {
        !self.is_in_progress()
    }
}
