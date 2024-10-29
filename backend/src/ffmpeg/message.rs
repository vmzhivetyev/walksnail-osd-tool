use ffmpeg_sidecar::event::FfmpegProgress;

pub enum FromFfmpegMessage {
    EncoderStartedWithCommand(String),
    DecoderFatalError(String),
    EncoderFatalError(String),
    EncoderProgress(FfmpegProgress),
    DecoderProgress(FfmpegProgress),
    DecoderFinished,
    EncoderFinished,
}

pub enum ToFfmpegMessage {
    AbortRender,
}
