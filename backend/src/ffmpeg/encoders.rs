use std::{fmt::Display, path::PathBuf, process::Command};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Codec {
    H264,
    H265,
    VP9,
    ProRes,
}

impl Display for Codec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Codec::H264 => write!(f, "H.264"),
            Codec::H265 => write!(f, "H.265 (HEVC)"),
            Codec::VP9 => write!(f, "VP9"),
            Codec::ProRes => write!(f, "Apple ProRes"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Encoder {
    pub name: String,
    pub codec: Codec,
    pub hardware: bool,
    pub detected: bool,
    pub constant_quality_args: Option<Vec<String>>,
    pub extra_args: Vec<String>,
}

impl Encoder {
    fn new(name: &str, codec: Codec, hardware: bool, constant_quality_args: Option<&[&str]>) -> Self {
        Self::new_with_extra_args(name, codec, hardware, constant_quality_args, &[])
    }

    fn new_with_extra_args(
        name: &str,
        codec: Codec,
        hardware: bool,
        constant_quality_args: Option<&[&str]>,
        extra_args: &[&str],
    ) -> Self {
        let constant_quality_args_vec = constant_quality_args.map(|args| args.iter().map(|&s| s.to_string()).collect());
        let extra_args_vec = extra_args.iter().map(|&s| s.to_string()).collect();

        Self {
            name: name.to_string(),
            codec,
            hardware,
            detected: false,
            constant_quality_args: constant_quality_args_vec,
            extra_args: extra_args_vec,
        }
    }

    #[tracing::instrument(ret)]
    pub fn get_available_encoders(ffmpeg_path: &PathBuf) -> Vec<Self> {
        // Apple QuickTime player on Mac supports hvc1. It doesn't support hev1 which is the default.
        // Make hevc videos be compatible with MacOS QuickTime.
        let hvc1tag = ["-tag:v", "hvc1"];

        #[rustfmt::skip]
        let mut all_encoders = [
            Encoder::new(
                "libx264", Codec::H264, false,
                Some(&["-crf", "19", "-b:v", "0k"])
            ),

            Encoder::new_with_extra_args(
                "libx265", Codec::H265, false,
                Some(&["-crf", "20", "-b:v", "0k"]),
                &hvc1tag
            ),

            #[cfg(target_os = "windows")]
            Encoder::new(
                "h264_amf", Codec::H264, true,
                None
            ),

            #[cfg(any(target_os = "windows", target_os = "linux"))]
            Encoder::new(
                "h264_nvenc", Codec::H264, true,
                Some(&["-qp", "18", "-b:v", "0k"])
            ),

            #[cfg(any(target_os = "windows", target_os = "linux"))]
            Encoder::new(
                "h264_qsv", Codec::H264, true,
                None
            ),

            #[cfg(target_os = "linux")]
            Encoder::new(
                "h264_vaapi", Codec::H264, true,
                None
            ),

            #[cfg(target_os = "linux")]
            Encoder::new(
                "h264_v4l2m2m", Codec::H264, true,
                None
            ),

            #[cfg(target_os = "macos")]
            Encoder::new(
                "h264_videotoolbox", Codec::H264, true,
                Some(&["-q:v", "70", "-b:v", "0k"]),
            ),

            #[cfg(target_os = "windows")]
            Encoder::new(
                "hevc_amf", Codec::H265, true,
                None
            ),

            #[cfg(any(target_os = "windows", target_os = "linux"))]
            Encoder::new_with_extra_args(
                "hevc_nvenc", Codec::H265, true,
                Some(&["-qp", "19", "-b:v", "0k"]),
                &hvc1tag
            ),

            #[cfg(any(target_os = "windows", target_os = "linux"))]
            Encoder::new(
                "hevc_qsv", Codec::H265, true,
                None
            ),

            #[cfg(target_os = "linux")]
            Encoder::new(
                "hevc_vaapi", Codec::H265, true,
                None
            ),

            #[cfg(target_os = "linux")]
            Encoder::new(
                "hevc_v4l2m2m", Codec::H265, true,
                None
            ),

            #[cfg(target_os = "macos")]
            Encoder::new_with_extra_args(
                "hevc_videotoolbox", Codec::H265, true,
                Some(&["-q:v", "70", "-b:v", "0k"]),
                &hvc1tag
            ),

            #[cfg(target_os = "macos")]
            Encoder::new(
                "libvpx-vp9", Codec::VP9, false,
                None
            ),

            #[cfg(target_os = "macos")]
            Encoder::new_with_extra_args(
                "prores_ks", Codec::ProRes, false,
                None,
                &["-profile:v", "4", "-pix_fmt", "yuva422p10le", "-alpha_bits", "8", "-vendor", "apl0"]
            ),

            #[cfg(target_os = "macos")]
            Encoder::new_with_extra_args(
                "prores_videotoolbox", Codec::ProRes, true,
                None,
                &["-profile:v", "4", "-pix_fmt", "yuva422p10le", "-alpha_bits", "8", "-vendor", "apl0"]),            
        ];

        all_encoders
            .par_iter_mut()
            .map(|encoder| {
                encoder.detected = Self::ffmpeg_encoder_available(encoder, ffmpeg_path);
                encoder.clone()
            })
            .collect()
    }

    fn ffmpeg_encoder_available(encoder: &Encoder, ffmpeg_path: &PathBuf) -> bool {
        let mut command = Command::new(ffmpeg_path);

        command
            .args([
                "-hide_banner",
                "-f",
                "lavfi",
                "-i",
                "nullsrc",
                "-c:v",
                &encoder.name,
                "-frames:v",
                "1",
                "-f",
                "null",
                "-",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        #[cfg(target_os = "windows")]
        std::os::windows::process::CommandExt::creation_flags(&mut command, crate::util::CREATE_NO_WINDOW);

        match command.status() {
            Ok(status) => status.success(),
            Err(_) => false,
        }
    }
}

impl Display for Encoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} — {} — {}",
            self.name,
            self.codec,
            if self.hardware { "hardware" } else { "software" }
        )
    }
}
