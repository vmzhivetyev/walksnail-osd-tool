use std::{
    env::current_exe,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use backend::{config::AppConfig, ffmpeg::VideoInfo, font::FontFile, osd::OsdFile, srt::SrtFile};
use egui::{FontFamily, FontId, Margin, RichText, Separator, TextStyle, Ui};
use github_release_check::{GitHubReleaseItem, LookupError};
use semver::Version;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{filter, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer};

use super::WalksnailOsdTool;
use crate::util::build_info::Build;

pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mov", "avi", "mkv", "wmv", "flv", "webm", "m4v", 
    "3gp", "3g2", "asf", "rm", "rmvb", "vob", "ogv", "drc",
    "mxf", "roq", "nsv", "f4v", "f4p", "f4a", "f4b"
];

pub const AVATAR_EXTENSIONS: &[&str] = &[
    "osd", "png", "srt"
];

impl WalksnailOsdTool {
    pub fn all_files_loaded(&self) -> bool {
        self.video_loaded() && self.osd_loaded() && self.font_loaded()
    }

    pub fn is_start_render_allowed(&self) -> bool {
        self.all_files_loaded() && self.is_encoder_selected() && self.is_output_file_path_allowed()
    }

    pub fn is_output_file_path_allowed(&self) -> bool {
        self.output_video_file != self.input_video_file
    }

    pub fn video_loaded(&self) -> bool {
        self.input_video_file.is_some() && self.video_info.is_some()
    }

    pub fn osd_loaded(&self) -> bool {
        self.osd_file.is_some()
    }

    pub fn srt_loaded(&self) -> bool {
        self.srt_file.is_some()
    }

    pub fn font_loaded(&self) -> bool {
        self.font_file.is_some()
    }

    pub fn is_encoder_selected(&self) -> bool {
        self.get_selected_encoder().is_some()
    }

    pub fn import_video_file(&mut self, file_handles: &[PathBuf]) {
        if let Some(video_file) = first_file_with_extentions(file_handles, VIDEO_EXTENSIONS) {
            self.input_video_file = Some(video_file.clone());
            self.video_info = VideoInfo::get(video_file, &self.dependencies.ffprobe_path).ok();

            // Generate default output file name for newly imported video file.
            self.ui_state.output_file_name = "".to_owned();
            self.update_output_video_path();

            // Try to load the matching OSD and SRT files
            self.import_osd_file(&[matching_file_with_extension(video_file, "osd")]);
            self.import_srt_file(&[matching_file_with_extension(video_file, "srt")]);
        }
    }

    pub fn import_osd_file(&mut self, file_handles: &[PathBuf]) {
        if let Some(osd_file_path) = filter_file_with_extention(file_handles, "osd") {
            self.osd_file = OsdFile::open(osd_file_path.clone()).ok();
            self.osd_preview.preview_frame = 1;
            self.osd_options.osd_playback_offset = 0.0;
            self.osd_options.character_size_class = None;
        }
    }

    pub fn import_srt_file(&mut self, file_handles: &[PathBuf]) {
        if let Some(str_file_path) = filter_file_with_extention(file_handles, "srt") {
            self.srt_file = SrtFile::open(str_file_path.clone()).ok();
            self.srt_options.show_distance &= self.srt_file.as_ref().map(|s| s.has_distance).unwrap_or(true);
            self.config_changed = Some(Instant::now());
        }
    }

    pub fn import_font_file(&mut self, file_handles: &[PathBuf]) {
        if let Some(font_file_path) = filter_file_with_extention(file_handles, "png") {
            self.font_file = FontFile::open(font_file_path.clone()).ok();
            self.config_changed = Some(Instant::now());
        }
    }
}

pub fn filter_file_with_extention<'a>(files: &'a [PathBuf], extention: &'a str) -> Option<&'a PathBuf> {
    files.iter().find_map(|f| {
        f.extension().and_then(|e| {
            if e.to_string_lossy() == extention {
                Some(f)
            } else {
                None
            }
        })
    })
}

pub fn first_file_with_extentions<'a>(files: &'a [PathBuf], extensions: &'a [&'a str]) -> Option<&'a PathBuf> {
    filter_files_with_extensions(files, extensions).first().copied()
}

// finds ALL files matching any of the provided extensions
pub fn filter_files_with_extensions<'a>(files: &'a [PathBuf], extensions: &'a [&'a str]) -> Vec<&'a PathBuf> {
    files.iter().filter_map(|f| {
        f.extension().and_then(|e| {
            let ext_str = e.to_string_lossy();
            if extensions.contains(&ext_str.as_ref()) {
                Some(f)
            } else {
                None
            }
        })
    }).collect()
}

#[tracing::instrument(ret, level = "info")]
pub fn matching_file_with_extension(path: &PathBuf, extention: &str) -> PathBuf {
    let file_name = path.file_stem().unwrap();
    let parent = path.parent().unwrap();
    parent.join(file_name).with_extension(extention)
}

pub fn separator_with_space(ui: &mut Ui, space: f32) {
    ui.scope(|ui| {
        ui.visuals_mut().widgets.noninteractive.bg_stroke.width = 0.5;
        ui.add(Separator::default().spacing(space));
    });
}

pub fn format_minutes_seconds(duration: &Duration) -> String {
    let minutes = duration.as_secs() / 60;
    let seconds = duration.as_secs() % 60;
    format!("{}:{:0>2}", minutes, seconds)
}

pub fn generate_default_output_file_name(input_file_path: &PathBuf) -> String {
    let input_file_name_no_ext = input_file_path
        .file_stem()
        .map_or("file".to_string(), |p| p.to_string_lossy().to_string());

    format!("{}_with_osd", input_file_name_no_ext)
}

pub fn generate_output_file_path(input_file_path: &Path, output_file_name: &String) -> PathBuf {
    let input_file_path_str = input_file_path.to_string_lossy();
    tracing::debug!("generate_output_file_path for {input_file_path_str}");

    let mut output_video_path: PathBuf = input_file_path.parent().unwrap().to_path_buf();
    let mut output_file_name = output_file_name.clone();
    if output_file_name.is_empty() {
        output_file_name = "file".to_owned();
    }
    output_video_path.push(&output_file_name);
    add_extension(&mut output_video_path, "mp4");
    output_video_path
}

fn add_extension(path: &mut std::path::PathBuf, extension: impl AsRef<std::path::Path>) {
    if path.extension() == Some(extension.as_ref().as_os_str()) {
        return;
    }

    match path.extension() {
        Some(ext) => {
            let mut ext = ext.to_os_string();
            ext.push(".");
            ext.push(extension.as_ref());
            path.set_extension(ext)
        }
        None => path.set_extension(extension.as_ref()),
    };
}

pub fn set_style(ctx: &egui::Context) {
    use egui::{
        FontFamily::{Monospace, Proportional},
        Style,
    };
    let mut style = Style::clone(&ctx.style());
    style.text_styles = [
        (TextStyle::Small, FontId::new(9.0, Proportional)),
        (TextStyle::Body, FontId::new(15.0, Proportional)),
        (TextStyle::Button, FontId::new(15.0, Proportional)),
        (TextStyle::Heading, FontId::new(17.0, Proportional)),
        (TextStyle::Monospace, FontId::new(14.0, Monospace)),
        (TextStyle::Name("Tooltip".into()), FontId::new(14.0, Proportional)),
    ]
    .into();
    style.spacing.window_margin = Margin {
        left: 20.0,
        right: 20.0,
        top: 6.0,
        bottom: 20.0,
    };
    ctx.set_style(style);
}

pub fn tooltip_text(text: &str) -> RichText {
    RichText::new(text).font(FontId::new(14.0, FontFamily::Proportional))
}

pub fn set_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "inter-regular".to_owned(),
        egui::FontData::from_static(include_bytes!("../../resources/fonts/Inter-Regular.ttf")),
    );

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "inter-regular".to_owned());

    ctx.set_fonts(fonts);
}

#[allow(clippy::from_over_into)]
impl Into<AppConfig> for &mut WalksnailOsdTool {
    fn into(self) -> AppConfig {
        AppConfig {
            osd_options: self.osd_options.clone(),
            srt_options: self.srt_options.clone(),
            render_options: self.render_settings.clone(),
            app_update: backend::util::AppUpdate {
                check_on_startup: self.app_update.check_on_startup,
            },
            dark_mode: self.dark_mode,
            font_path: self
                .font_file
                .as_ref()
                .map(|f| f.file_path.clone())
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        }
    }
}

pub fn init_tracing() -> Option<WorkerGuard> {
    directories::ProjectDirs::from("rs", "", "walksnail-osd-tool").map(|dir| {
        let log_dir = dir.data_dir();

        std::fs::remove_file(log_dir.join("walksnail-osd-tool.log")).ok();

        let file_appender = tracing_appender::rolling::never(log_dir, "walksnail-osd-tool.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let stdout_log = tracing_subscriber::fmt::layer()
            .pretty()
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            .with_filter(filter::LevelFilter::INFO);
        let file_log = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .compact()
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            .with_writer(non_blocking)
            .with_filter(filter::LevelFilter::INFO);
        tracing_subscriber::registry().with(stdout_log).with(file_log).init();

        guard
    })
}

pub fn get_dependency_path(dependency: &str) -> PathBuf {
    let cur_exe = current_exe().unwrap();
    let exe_dir = cur_exe.parent().unwrap();

    if cfg!(all(target_os = "macos", feature = "macos-app-bundle")) {
        // Folder structure:
        // |
        // +-- MacOS
        //     +-- walksnail-osd-tool
        //     +-- ffmpeg
        //     +-- ffprobe
        exe_dir.join(dependency)
    } else if cfg!(all(target_os = "windows", feature = "windows-installer")) {
        // Folder structure:
        // |
        // +-- bin
        // |   +-- walksnail-osd-tool.exe
        // +-- ffmpeg
        //     +-- ffmpeg.exe
        //     +-- ffprobe.exe
        exe_dir.parent().unwrap().join("ffmpeg").join(dependency)
    } else {
        dependency.into()
    }
}

pub mod build_info {
    use std::fmt::Display;

    use semver::Version;

    pub enum Build {
        #[allow(dead_code)]
        Release {
            version: Version,
            commit: String,
        },
        Dev {
            commit: String,
        },
        Unknown,
    }

    impl Display for Build {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Build::Release { version, .. } => write!(f, "{version}"),
                Build::Dev { commit } => write!(f, "dev ({commit})"),
                Build::Unknown => write!(f, "Unknown"),
            }
        }
    }

    pub fn get_version() -> Build {
        let version: Option<Version> = option_env!("GIT_VERSION").and_then(|s| Version::parse(s).ok());
        let short_hash: Option<&'static str> = option_env!("GIT_COMMIT_HASH");

        match (version, short_hash.map(|s| s.to_string())) {
            (Some(version), Some(commit)) => Build::Release { version, commit },
            (None, Some(commit)) => Build::Dev { commit },
            _ => Build::Unknown,
        }
    }

    pub fn get_compiler() -> &'static str {
        env!("VERGEN_RUSTC_SEMVER")
    }

    pub fn get_target() -> &'static str {
        env!("VERGEN_CARGO_TARGET_TRIPLE")
    }
}

#[tracing::instrument(ret)]
pub fn check_updates() -> Result<Option<GitHubReleaseItem>, LookupError> {
    if let Build::Release {
        version: current_version,
        ..
    } = build_info::get_version()
    {
        let github = github_release_check::GitHub::new().unwrap();
        let releases = github.query("vmzhivetyev/walksnail-osd-tool")?;
        let update_target = releases
            .iter()
            .find(|release| {
                Version::parse(release.tag_name.trim_start_matches('v'))
                    .map_or(false, |version| should_update_to_version(&current_version, &version))
            })
            .cloned();
        Ok(update_target)
    } else {
        Ok(None)
    }
}

fn should_update_to_version(current_version: &Version, to_version: &Version) -> bool {
    if to_version <= current_version {
        return false;
    }

    let version_is_full_release = to_version.pre.is_empty();
    if version_is_full_release {
        return true;
    }

    let current_version_is_prerelease = !current_version.pre.is_empty();
    if current_version_is_prerelease {
        return to_version.major == current_version.major && to_version.minor == current_version.minor;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(version: &str) -> Version {
        Version::parse(version).unwrap()
    }

    #[test]
    fn update_to_new_release() {
        let current_version = version("0.1.0");
        let new_version = version("0.2.0");
        assert!(should_update_to_version(&current_version, &new_version));
    }

    #[test]
    fn not_update_to_older_release() {
        let current_version = version("0.2.0");
        let new_version = version("0.1.0");
        assert!(!should_update_to_version(&current_version, &new_version));
    }

    #[test]
    fn update_from_prerelease_to_full_release() {
        let current_version = version("0.1.0-beta.2");
        let new_version = version("0.1.0");
        assert!(should_update_to_version(&current_version, &new_version));
    }

    #[test]
    fn update_from_prerelease_to_new_prerelease() {
        let current_version = version("0.1.0-beta.1");
        let new_version = version("0.1.0-beta.3");
        assert!(should_update_to_version(&current_version, &new_version));
    }

    #[test]
    fn not_update_from_prerelease_to_older_prerelease() {
        let current_version = version("0.1.0-beta.3");
        let new_version = version("0.1.0-beta.2");
        assert!(!should_update_to_version(&current_version, &new_version));
    }

    #[test]
    fn not_update_from_prerelease_to_prerelease_in_new_cyce() {
        let current_version = version("0.1.0-beta.3");
        let new_version = version("0.2.0-beta.2");
        assert!(!should_update_to_version(&current_version, &new_version));
    }

    #[test]
    fn not_update_from_release_to_prerelease_of_new_release() {
        let current_version = version("0.1.0");
        let new_version = version("0.2.0-beta.2");
        assert!(!should_update_to_version(&current_version, &new_version));
    }

    #[test]
    fn not_update_to_same_release() {
        let current_version = version("0.1.0");
        assert!(!should_update_to_version(&current_version, &current_version));
    }
}
