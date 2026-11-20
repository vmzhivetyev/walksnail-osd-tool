use std::{
    io,
    path::PathBuf,
    process::{Command, Stdio},
};

use image::RgbaImage;

/// Decode the single video frame shown at `time_secs`, for the preview.
pub fn extract_video_frame(
    ffmpeg_path: &PathBuf,
    video: &PathBuf,
    time_secs: f32,
    width: u32,
    height: u32,
) -> io::Result<RgbaImage> {
    let mut command = Command::new(ffmpeg_path);

    command
        .args(["-hide_banner", "-loglevel", "error", "-ss"])
        .arg(format!("{:.3}", time_secs.max(0.0)))
        .arg("-i")
        .arg(video)
        .args(["-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgba", "-"])
        .stdin(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(target_os = "windows")]
    std::os::windows::process::CommandExt::creation_flags(&mut command, crate::util::CREATE_NO_WINDOW);

    let output = command.output()?;

    RgbaImage::from_raw(width, height, output.stdout).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("ffmpeg returned no {width}x{height} frame at {time_secs:.3}s"),
        )
    })
}
