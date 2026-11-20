use ffmpeg_sidecar::event::FfmpegProgress;

pub enum FromFfmpegMessage {
    EncoderFatalError(String),
    EncoderProgress(FfmpegProgress),
    EncoderFinished,
}

pub enum ToFfmpegMessage {
    AbortRender,
}
