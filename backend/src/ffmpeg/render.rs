use std::{
    io::{self, Read, Write},
    path::PathBuf,
    sync::Arc,
    thread,
};

use crossbeam_channel::{Receiver, Sender};
use ffmpeg_sidecar::{
    child::FfmpegChild,
    command::FfmpegCommand,
    event::{FfmpegEvent, LogLevel},
};
use image::RgbaImage;

use super::{render_settings::RenderSettings, Encoder, FromFfmpegMessage, ToFfmpegMessage, VideoInfo};
use crate::{
    font,
    osd::{self, OsdOptions},
    overlay::OverlayFrameIter,
    srt::{self, SrtOptions},
};

/// Overlay frames are fed to ffmpeg at no more than this rate. OSD and SRT data update far slower
/// than this, and every overlay frame costs a full RGBA frame through the pipe.
const MAX_OVERLAY_FRAME_RATE: f32 = 30.0;

/// The live view during a render: the encoder hands over one composited frame per second.
/// `split` shares frames instead of copying them, so this only costs one RGBA conversion per second.
const LIVE_VIEW_FRAME_RATE: u32 = 1;

/// Pick the overlay frame rate for a video: the video frame rate divided by the smallest integer that
/// brings it to `MAX_OVERLAY_FRAME_RATE` or below, so overlay frames stay aligned with video frames.
fn overlay_frame_rate(video_frame_rate: f32) -> f32 {
    let divisor = (video_frame_rate / MAX_OVERLAY_FRAME_RATE).ceil().max(1.0);
    video_frame_rate / divisor
}

fn run_overlay_frames_to_queue(
    frame_iter: impl Iterator<Item = Arc<RgbaImage>>,
    tx: Sender<Arc<RgbaImage>>,
    abort_rx: Receiver<ToFfmpegMessage>,
) {
    for frame in frame_iter {
        // Stopping the overlay stream makes ffmpeg end the output (`eof_action=endall`), which also
        // finalizes the partially rendered file.
        if let Ok(ToFfmpegMessage::AbortRender) = abort_rx.try_recv() {
            tracing::info!("Render aborted");
            break;
        }
        // If send fails (receiver dropped), we exit the thread
        if tx.send(frame).is_err() {
            break;
        }
    }
}

/// `frame_to_ui_tx` is only set when the overlay frames are the whole output video (chroma key);
/// otherwise the live view comes from the encoder.
fn run_overlay_frames_from_queue_to_encoder(
    rx: Receiver<Arc<RgbaImage>>,
    mut encoder_stdin: impl Write,
    frame_to_ui_tx: Option<Sender<RgbaImage>>,
) {
    let mut last_frame_sent_to_ui: Option<Arc<RgbaImage>> = None;

    while let Ok(frame) = rx.recv() {
        // write_all blocks while the encoder is not ready to read its stdin, i.e. when the encoder is the bottleneck.
        if let Err(e) = encoder_stdin.write_all(frame.as_raw()) {
            // The encoder closed its input: it either finished or failed, and reports that itself.
            tracing::warn!("Encoder stopped accepting overlay frames: {}", e);
            break;
        }

        if let Some(frame_to_ui_tx) = &frame_to_ui_tx {
            let already_shown = last_frame_sent_to_ui
                .as_ref()
                .is_some_and(|last| Arc::ptr_eq(last, &frame));
            if !already_shown && frame_to_ui_tx.is_empty() {
                let _ = frame_to_ui_tx.try_send(frame.as_ref().clone());
                last_frame_sent_to_ui = Some(frame);
            }
        }
    }
}

/// Read the live view frames the encoder writes to its stdout and pass them to the UI.
/// Frames the UI has not caught up with are dropped; stdout is always drained so the encoder never blocks on it.
fn run_live_view_frames_to_ui(mut encoder_stdout: impl Read, width: u32, height: u32, frame_to_ui_tx: Sender<RgbaImage>) {
    let frame_size = width as usize * height as usize * 4;
    loop {
        let mut buffer = vec![0u8; frame_size];
        if encoder_stdout.read_exact(&mut buffer).is_err() {
            break; // encoder finished
        }
        if let Some(image) = RgbaImage::from_raw(width, height, buffer) {
            let _ = frame_to_ui_tx.try_send(image);
        }
    }
}

#[allow(clippy::type_complexity)]
#[tracing::instrument(skip(osd_frames, srt_frames, font_file), err)]
pub fn start_video_render(
    ffmpeg_path: &PathBuf,
    input_video: &PathBuf,
    output_video: &PathBuf,
    osd_frames: Vec<osd::Frame>,
    srt_frames: Option<Vec<srt::SrtFrame>>,
    font_file: font::FontFile,
    srt_font: rusttype::Font<'static>,
    osd_options: &OsdOptions,
    srt_options: &SrtOptions,
    video_info: &VideoInfo,
    render_settings: &RenderSettings,
    encoder: &Encoder,
) -> Result<
    (
        Sender<ToFfmpegMessage>,
        Receiver<FromFfmpegMessage>,
        Receiver<RgbaImage>,
    ),
    io::Error,
> {
    if input_video == output_video {
        panic!("This should never happen");
    }

    let chroma_key = if render_settings.use_chroma_key {
        Some(render_settings.chroma_key)
    } else {
        None
    };

    // With a chroma key the overlay frames are the whole output video, so they are generated at the
    // video frame rate. Otherwise they are composited onto the original video by ffmpeg.
    let (overlay_rate, overlay_frame_count) = if chroma_key.is_some() {
        (video_info.frame_rate, video_info.total_frames)
    } else {
        let rate = overlay_frame_rate(video_info.frame_rate);
        // One extra frame so the overlay stream never ends before the video; ffmpeg stops at the end of the video.
        let count = (video_info.duration.as_secs_f32() * rate).ceil() as u32 + 1;
        (rate, count)
    };

    let live_view_from_encoder = render_settings.rendering_live_view && chroma_key.is_none();

    let mut encoder_process = spawn_encoder(
        ffmpeg_path,
        video_info.width,
        video_info.height,
        overlay_rate,
        video_info.time_base,
        render_settings.bitrate_mbps,
        render_settings.keep_quality,
        encoder,
        output_video,
        render_settings.upscale,
        render_settings.rescale_to_4x3_aspect,
        chroma_key,
        input_video,
        live_view_from_encoder,
    )?;

    // Channels to communicate with ffmpeg handler thread
    let (from_ffmpeg_tx, from_ffmpeg_rx) = crossbeam_channel::unbounded();
    let (to_ffmpeg_tx, to_ffmpeg_rx) = crossbeam_channel::unbounded();
    let (frames_for_ui_tx, frames_for_ui_rx) = crossbeam_channel::bounded(1);

    let overlay_frame_iter = OverlayFrameIter::new(
        overlay_rate,
        overlay_frame_count,
        video_info.width,
        video_info.height,
        osd_frames,
        srt_frames,
        font_file,
        srt_font,
        osd_options,
        srt_options,
        chroma_key,
    );

    let encoder_stdin = encoder_process.take_stdin().expect("Failed to get `stdin` for encoder");

    // Unchanged overlay frames are the same shared buffer, so the queue costs little memory.
    let (overlay_queue_in, overlay_queue_out) = crossbeam_channel::bounded::<Arc<RgbaImage>>(16);

    thread::Builder::new()
        .name("Overlay frames to queue".into())
        .spawn(move || {
            tracing::info_span!("overlay frames iter -> queue").in_scope(|| {
                run_overlay_frames_to_queue(overlay_frame_iter, overlay_queue_in, to_ffmpeg_rx);
            });
        })
        .expect("Failed to spawn producer thread");

    let (overlay_frames_for_ui_tx, encoder_frames_for_ui_tx) = match (render_settings.rendering_live_view, live_view_from_encoder) {
        (false, _) => (None, None),
        (true, true) => (None, Some(frames_for_ui_tx)),
        (true, false) => (Some(frames_for_ui_tx), None),
    };

    thread::Builder::new()
        .name("Overlay frames from queue to encoder".into())
        .spawn(move || {
            tracing::info_span!("overlay frames queue -> encoder").in_scope(|| {
                run_overlay_frames_from_queue_to_encoder(overlay_queue_out, encoder_stdin, overlay_frames_for_ui_tx);
            });
        })
        .expect("Failed to spawn consumer thread");

    // Read live view frames from the encoder's stdout ourselves. With stdout taken, ffmpeg-sidecar only parses stderr:
    // its stdout reader picks the frame format by output order, which breaks with a file output plus a stdout output.
    if let Some(frame_to_ui_tx) = encoder_frames_for_ui_tx {
        let encoder_stdout = encoder_process.take_stdout().expect("Failed to get `stdout` for encoder");
        let (width, height) = (video_info.width, video_info.height);
        thread::Builder::new()
            .name("Live view frames to UI".into())
            .spawn(move || run_live_view_frames_to_ui(encoder_stdout, width, height, frame_to_ui_tx))
            .expect("Failed to spawn live view thread");
    }

    // On yet another thread run the encoder to completion
    thread::Builder::new()
        .name("Encoder handler".into())
        .spawn(move || {
            tracing::info_span!("Encoder handler thread").in_scope(|| {
                encoder_process
                    .iter()
                    .expect("Failed to create encoder iterator")
                    .for_each(|event| handle_encoder_events(event, &from_ffmpeg_tx));
            });
        })
        .expect("Failed to spawn encoder handler thread");

    Ok((to_ffmpeg_tx, from_ffmpeg_rx, frames_for_ui_rx))
}

/// Spawn the ffmpeg process that produces the output video.
///
/// Its stdin receives raw RGBA overlay frames at `overlay_frame_rate`:
/// - without a chroma key, ffmpeg decodes `original_file` itself and composites the overlay onto it;
/// - with a chroma key, the overlay frames already contain the background and are encoded as they are.
///
/// With `live_view` (only used without a chroma key), a copy of the composited video at a low frame rate is also
/// written to stdout as raw RGBA frames for the UI.
#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(ffmpeg_path))]
pub fn spawn_encoder(
    ffmpeg_path: &PathBuf,
    width: u32,
    height: u32,
    overlay_frame_rate: f32,
    time_base: u32,
    bitrate_mbps: u32,
    keep_quality: bool,
    video_encoder: &Encoder,
    output_video: &PathBuf,
    upscale: bool,
    rescale_to_4x3_aspect: bool,
    chroma_key: Option<[f32; 4]>,
    original_file: &PathBuf,
    live_view: bool,
) -> Result<FfmpegChild, io::Error> {
    let mut encoder_command = FfmpegCommand::new_with_path(ffmpeg_path);
    let mut output_video = output_video.clone();
    let is_nvenc = video_encoder.name.contains("nvenc");

    encoder_command.create_no_window();

    if chroma_key.is_some() {
        encoder_command
            .format("rawvideo")
            .pix_fmt("rgba")
            .size(width, height)
            .rate(overlay_frame_rate)
            .input("-");

        encoder_command
            .input(original_file.to_str().unwrap())
            .map("0")
            .map("1:a?")
            .codec_audio("copy");

        if upscale {
            if is_nvenc {
                encoder_command.args(["-vf", "format=rgb24,hwupload_cuda,scale_cuda=-2:1440:3"]);
            } else {
                encoder_command.args(["-vf", "scale=-2:1440:flags=bicubic"]);
            }
        } else if is_nvenc {
            encoder_command.args(["-vf", "format=rgb24,hwupload_cuda"]);
        }
    } else {
        encoder_command
            .args(["-hwaccel", "auto"])
            .input(original_file.to_str().unwrap());

        encoder_command
            .format("rawvideo")
            .pix_fmt("rgba")
            .size(width, height)
            .rate(overlay_frame_rate)
            .input("-");

        // `eof_action=endall` ends the output when the overlay stream ends, which is how a render is aborted.
        // The overlay stream is one frame longer than the video, so a full render ends with the video.
        // `format=yuv420` keeps the composited video in the source's YUV 4:2:0 instead of letting format
        // negotiation pick RGBA because of the live view branch.
        let composite = "[0:v][1:v]overlay=eof_action=endall:format=yuv420";
        let output_filter = match (upscale, is_nvenc) {
            (true, true) => "format=yuv420p,hwupload_cuda,scale_cuda=-2:1440:3",
            (true, false) => "scale=-2:1440:flags=bicubic",
            (false, _) => "null",
        };
        let filter_graph = if live_view {
            format!(
                "{composite},split=2[composited][live];[composited]{output_filter}[out];\
                 [live]fps={LIVE_VIEW_FRAME_RATE},format=rgba[live_view]"
            )
        } else {
            format!("{composite},{output_filter}[out]")
        };
        encoder_command.filter_complex(filter_graph);

        encoder_command
            .map("[out]")
            .map("0:a?")
            .codec_audio("copy");
    }

    if is_nvenc {
        // p5 measured nearly 2x faster than p7 for HEVC (1.5x for AV1) at the same file size,
        // with a PSNR difference of 0.04 dB or less.
        encoder_command.args(["-preset", "p5", "-tune", "hq"]);
    }

    if rescale_to_4x3_aspect {
        // It will affect the aspect ratio stored at container level without affecting final video resolution.
        // Example ffprobe of a final video: "... 1280x720 [SAR 3:4 DAR 4:3] ...".
        // Such video will be played back the same way as if it really was 4:3.
        encoder_command.args(["-aspect", "4:3"]);
    }

    encoder_command.codec_video(&video_encoder.name);

    if keep_quality {
        // this will crash when encoder is not setup to support constant quality mode.
        let args = &video_encoder.constant_quality_args.clone().unwrap();
        encoder_command.args(args);
    } else {
        encoder_command.args(["-b:v", &format!("{}M", bitrate_mbps)]);
    }

    encoder_command
        .args(&video_encoder.extra_args)
        .args(["-video_track_timescale", time_base.to_string().as_str()]);

    if &video_encoder.name == "prores_ks" {
        output_video.set_extension("mov");
    } else {
        if chroma_key.is_none() && !is_nvenc {
            encoder_command.pix_fmt("yuv420p");
        }
    }

    encoder_command.overwrite().output(output_video.to_str().unwrap());

    // Live view frames go to stdout as raw RGBA, read by `run_live_view_frames_to_ui`. It must come after the
    // video file: ffmpeg's progress (frame count, fps, bitrate) describes output #0.
    if live_view && chroma_key.is_none() {
        encoder_command
            .map("[live_view]")
            .format("rawvideo")
            .pix_fmt("rgba")
            .output("-");
    }

    tracing::info!(
        "✅✅✅✅✅✅✅ {}",
        crate::util::command_to_cli(encoder_command.as_inner())
    );

    let encoder = encoder_command.spawn()?;
    Ok(encoder)
}

fn handle_encoder_events(ffmpeg_event: FfmpegEvent, ffmpeg_sender: &Sender<FromFfmpegMessage>) {
    match ffmpeg_event {
        FfmpegEvent::ParsedVersion(version) => {
            println!("ffmpeg encoder >>> FFmpeg version: {:?}", version);
        }
        FfmpegEvent::ParsedConfiguration(config) => {
            println!("ffmpeg encoder >>> FFmpeg configuration: {:?}", config);
        }
        FfmpegEvent::ParsedStreamMapping(mapping) => {
            println!("ffmpeg encoder >>> Stream mapping: {}", mapping);
        }
        FfmpegEvent::ParsedInput(input) => {
            println!("ffmpeg encoder >>> Input details: {:?}", input);
        }
        FfmpegEvent::ParsedOutput(output) => {
            println!("ffmpeg encoder >>> Output details: {:?}", output);
        }
        FfmpegEvent::ParsedInputStream(stream) => {
            println!("ffmpeg encoder >>> Input stream: {:?}", stream);
        }
        FfmpegEvent::ParsedOutputStream(stream) => {
            println!("ffmpeg encoder >>> Output stream: {:?}", stream);
        }
        FfmpegEvent::ParsedDuration(duration) => {
            println!("ffmpeg encoder >>> Duration: {:?}", duration);
        }
        FfmpegEvent::Progress(p) => {
            ffmpeg_sender.send(FromFfmpegMessage::EncoderProgress(p)).ok();
        }
        FfmpegEvent::Log(level, e) => {
            if level == LogLevel::Fatal
            // there are some fatal errors that ffmpeg considers normal errors
            || e.contains("Error initializing output stream")
            || e.contains("[error] Cannot load")
            {
                tracing::error!("ffmpeg fatal error: {}", &e);
                ffmpeg_sender.send(FromFfmpegMessage::EncoderFatalError(e)).ok();
            } else {
                println!("ffmpeg encoder >>> {}", e);
            }
        }
        FfmpegEvent::LogEOF => {
            tracing::warn!("ffmpeg encoder EOF reached");
            ffmpeg_sender.send(FromFfmpegMessage::EncoderFinished).ok();
        }
        _ => {}
    }
}
