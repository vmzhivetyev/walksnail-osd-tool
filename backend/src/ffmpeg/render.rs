use std::{
    io::{self, Write}, path::PathBuf, thread
};

use crossbeam_channel::{Receiver, Sender};
use ffmpeg_sidecar::{
    child::FfmpegChild,
    command::FfmpegCommand,
    event::{FfmpegEvent, LogLevel},
};

use super::{render_settings::RenderSettings, Encoder, FromFfmpegMessage, ToFfmpegMessage, VideoInfo};
use crate::{
    font,
    osd::{self, OsdOptions},
    overlay::FrameOverlayIter,
    srt::{self, SrtOptions},
};

fn run_ready_frames_to_queue(frame_iter: impl Iterator<Item = ffmpeg_sidecar::event::OutputVideoFrame>, tx: Sender<ffmpeg_sidecar::event::OutputVideoFrame>) {
    for frame in frame_iter {
        // If send fails (receiver dropped), we exit the thread
        if tx.send(frame).is_err() {
            break;
        }
        // tracing::debug!("Frame queued for processing. Queued: {}", tx.len());
    }
}

fn run_ready_frames_from_queue_to_encoder(rx: Receiver<ffmpeg_sidecar::event::OutputVideoFrame>, mut encoder_stdin: impl Write) {
    while let Ok(frame) = rx.recv() {
        let _start = std::time::Instant::now();

        // write_all can take a lot of time if the encoder process is not ready to read it's stdin, it means encoder is the bottleneck.
        // If write fails, we just log it and continue
        if let Err(e) = encoder_stdin.write_all(&frame.data) {
            tracing::error!("Failed to write frame: {}", e);
            continue;
        }
        
        // tracing::info!(
        //     "encoder_stdin.write_all done in {:?}.",
        //     _start.elapsed()
        // );
    }
}

#[tracing::instrument(skip(osd_frames, srt_frames, font_file), err)]
pub fn start_video_render(
    ffmpeg_path: &PathBuf,
    input_video: &PathBuf,
    output_video: &PathBuf,
    osd_frames: Vec<osd::Frame>,
    srt_frames: Vec<srt::SrtFrame>,
    font_file: font::FontFile,
    srt_font: rusttype::Font<'static>,
    osd_options: &OsdOptions,
    srt_options: &SrtOptions,
    video_info: &VideoInfo,
    render_settings: &RenderSettings,
) -> Result<(Sender<ToFfmpegMessage>, Receiver<FromFfmpegMessage>), io::Error> {
    let mut decoder_process = spawn_decoder(ffmpeg_path, input_video)?;

    let mut encoder_process = spawn_encoder(
        ffmpeg_path,
        video_info.width,
        video_info.height,
        video_info.frame_rate,
        video_info.time_base,
        render_settings.bitrate_mbps,
        render_settings.keep_quality,
        &render_settings.encoder,
        output_video,
        render_settings.upscale,
        render_settings.rescale_to_4x3_aspect,
        if render_settings.use_chroma_key {
            Some(render_settings.chroma_key)
        } else {
            None
        },
        input_video,
    )?;

    // Channels to communicate with ffmpeg handler thread
    let (from_ffmpeg_tx, from_ffmpeg_rx) = crossbeam_channel::unbounded();
    let (to_ffmpeg_tx, to_ffmpeg_rx) = crossbeam_channel::unbounded();

    // Iterator over decoded video and OSD frames
    let frame_overlay_iter = FrameOverlayIter::new(
        decoder_process
            .iter()
            .expect("Failed to create `FfmpegIterator` for decoder"),
        decoder_process,
        osd_frames,
        srt_frames,
        font_file,
        srt_font,
        osd_options,
        srt_options,
        from_ffmpeg_tx.clone(),
        to_ffmpeg_rx,
        if render_settings.use_chroma_key {
            Some(render_settings.chroma_key)
        } else {
            None
        },
    );

    // On another thread run the decoder iterator to completion and feed the output to the encoder's stdin
    let encoder_stdin = encoder_process.take_stdin().expect("Failed to get `stdin` for encoder");

    let (ready_frames_queue_in, ready_frames_queue_out) = crossbeam_channel::bounded:: <ffmpeg_sidecar::event::OutputVideoFrame>(100);

    thread::Builder::new()
        .name("Push ready frames to queue".into())
        .spawn(move || {
            tracing::info_span!("ready frames iter -> queue").in_scope(|| {
                run_ready_frames_to_queue(frame_overlay_iter, ready_frames_queue_in);
            });
        })
        .expect("Failed to spawn producer thread");

    thread::Builder::new()
        .name("Pop ready frames from queue to encoder".into())
        .spawn(move || {
            tracing::info_span!("ready frames queue -> encoder").in_scope(|| {
                run_ready_frames_from_queue_to_encoder(ready_frames_queue_out, encoder_stdin);
            });
        })
        .expect("Failed to spawn consumer thread");

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

    Ok((to_ffmpeg_tx, from_ffmpeg_rx))
}

#[tracing::instrument(skip(ffmpeg_path))]
pub fn spawn_decoder(ffmpeg_path: &PathBuf, input_video: &PathBuf) -> Result<FfmpegChild, io::Error> {
    let decoder = FfmpegCommand::new_with_path(ffmpeg_path)
        .create_no_window()
        .input(input_video.to_str().unwrap())
        .args(["-f", "rawvideo", "-pix_fmt", "rgba", "-"])
        .spawn()?;
    Ok(decoder)
}

#[tracing::instrument(skip(ffmpeg_path))]
pub fn spawn_encoder(
    ffmpeg_path: &PathBuf,
    width: u32,
    height: u32,
    frame_rate: f32,
    time_base: u32,
    bitrate_mbps: u32,
    keep_quality: bool,
    video_encoder: &Encoder,
    output_video: &PathBuf,
    upscale: bool,
    rescale_to_4x3_aspect: bool,
    chroma_key: Option<[f32; 4]>,
    original_file: &PathBuf,
) -> Result<FfmpegChild, io::Error> {
    let mut encoder_command = FfmpegCommand::new_with_path(ffmpeg_path);
    let mut output_video = output_video.clone();

    encoder_command
        .create_no_window()
        .format("rawvideo")
        .pix_fmt("rgba")
        .size(width, height)
        .rate(frame_rate)
        .input("-");

    encoder_command
        .input(original_file.to_str().unwrap())
        .map("0")
        .map("1:a?")
        .codec_audio("copy");

    if upscale {
        if video_encoder.name.contains("nvenc") {
            encoder_command.args(["-vf", "format=rgb24,hwupload_cuda,scale_cuda=-2:1440:3"]);
        } else {
            encoder_command.args(["-vf", "scale=-2:1440:flags=bicubic"]);
        }
    } else {
        if video_encoder.name.contains("nvenc") {
            encoder_command.args(["-vf", "format=rgb24,hwupload_cuda"]);
        }
    }

    if rescale_to_4x3_aspect {
        // It will affect the aspect ratio stored at container level without affecting final video resolution.
        // Example ffprobe of a final video: "... 1280x720 [SAR 3:4 DAR 4:3] ...".
        // Such video will be played back the same way as if it really was 4:3.
        encoder_command.args(["-aspect", "4:3"]);
    }
    
    encoder_command
        .codec_video(&video_encoder.name);

    if keep_quality {
        if video_encoder.name.contains("hevc_nvenc") {
            encoder_command
                .args(["-rc", "constqp"])
                .args(["-qp", "27"])
                .args(["-b:v", "0k"]);
        }
        else if video_encoder.name.contains("h264_nvenc") {
            encoder_command
                .args(["-rc", "constqp"])
                .args(["-qp", "22"])
                .args(["-b:v", "0k"]);
        }
        else if video_encoder.name.contains("libx265") {
            encoder_command
                .args(["-crf", "29"])
                .args(["-b:v", "0k"]);
        }
        else if video_encoder.name.contains("libx264") {
            encoder_command
                .args(["-crf", "23"])
                .args(["-b:v", "0k"]);
        }
        else {
            encoder_command
                .args(["-b:v", &format!("{}M", bitrate_mbps)]);
        }
    }
    else {
        encoder_command
            .args(["-b:v", &format!("{}M", bitrate_mbps)]);
    }

    encoder_command
        .args(&video_encoder.extra_args)
        .args(["-video_track_timescale", time_base.to_string().as_str()]);      
        
    // if let Some(chroma_color) = chroma_key {
    //     if chroma_color[3] > 0.99 {
    //         encoder_command
    //             .pix_fmt("yuv420p");
    //     } else {
    //         encoder_command
    //             .pix_fmt("yuva420p")
    //             .args(["-alpha_quality", "1", "-f", "webm"]);

    //         output_video.set_extension("webm");
    //     }
    // } else {
    //     encoder_command
    //         .pix_fmt("yuv420p");
    // }

    if &video_encoder.name == "prores_ks" {
        output_video.set_extension("mov");
    }

    encoder_command.overwrite().output(output_video.to_str().unwrap());

    tracing::info!(
        "✅✅✅✅✅✅✅ {}",
        crate::util::command_to_cli(encoder_command.as_inner())
    );

    let encoder = encoder_command.spawn()?;
    Ok(encoder)
}

fn handle_encoder_events(ffmpeg_event: FfmpegEvent, ffmpeg_sender: &Sender<FromFfmpegMessage>) {
    match ffmpeg_event {
        FfmpegEvent::Log(level, e) => {
            if level == LogLevel::Fatal
            // there are some fatal errors that ffmpeg considers normal errors
            || e.contains("Error initializing output stream")
            || e.contains("[error] Cannot load")
            {
                tracing::error!("ffmpeg fatal error: {}", &e);
                ffmpeg_sender.send(FromFfmpegMessage::EncoderFatalError(e)).unwrap();
            }
        }
        FfmpegEvent::LogEOF => {
            tracing::warn!("ffmpeg encoder EOF reached");
            ffmpeg_sender.send(FromFfmpegMessage::EncoderFinished).ok();
        }
        _ => {}
    }
}

pub fn handle_decoder_events(ffmpeg_event: FfmpegEvent, ffmpeg_sender: &Sender<FromFfmpegMessage>) {
    match ffmpeg_event {
        FfmpegEvent::Progress(p) => {
            ffmpeg_sender.send(FromFfmpegMessage::Progress(p)).unwrap();
        }
        FfmpegEvent::Done | FfmpegEvent::LogEOF => {
            ffmpeg_sender.send(FromFfmpegMessage::DecoderFinished).unwrap();
        }
        FfmpegEvent::Log(LogLevel::Fatal, e) => {
            tracing::error!("ffmpeg fatal error: {}", &e);
            ffmpeg_sender.send(FromFfmpegMessage::DecoderFatalError(e)).unwrap();
        }
        FfmpegEvent::Log(LogLevel::Warning | LogLevel::Error, e) => {
            tracing::warn!("ffmpeg log: {}", e);
        }
        _ => {}
    }
}
