use image::{Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;

use crate::srt::{SrtFrameData, SrtDebugFrameData, SrtOptions};

#[inline]
pub fn overlay_srt_data(
    image: &mut RgbaImage,
    srt_data: &SrtFrameData,
    font: &rusttype::Font,
    srt_options: &SrtOptions,
) {
    let time_str = if srt_options.show_time {
        let minutes = srt_data.flight_time / 60;
        let seconds = srt_data.flight_time % 60;
        format!("Time:{}:{:0>2}  ", minutes, seconds % 60)
    } else {
        "".into()
    };

    let sbat_str = if srt_options.show_sbat {
        format!("SBat:{: >4.1}V  ", srt_data.sky_bat)
    } else {
        "".into()
    };

    let gbat_str = if srt_options.show_gbat {
        format!("GBat:{: >4.1}V  ", srt_data.ground_bat)
    } else {
        "".into()
    };

    let signal_str = if srt_options.show_signal {
        format!("Signal:{}  ", srt_data.signal)
    } else {
        "".into()
    };

    let latency_str = if srt_options.show_latency {
        format!("Latency:{: >3}ms  ", srt_data.latency)
    } else {
        "".into()
    };

    let bitrate_str = if srt_options.show_bitrate {
        format!("Bitrate:{: >4.1}Mbps  ", srt_data.bitrate_mbps)
    } else {
        "".into()
    };

    let distance_str = if srt_options.show_distance {
        let distance = srt_data.distance;
        if distance > 999 {
            let km = distance as f32 / 1000.0;
            format!("Distance:{:.2}km", km)
        } else {
            format!("Distance:{: >3}m", srt_data.distance)
        }
    } else {
        "".into()
    };

    let srt_string = format!("{time_str}{sbat_str}{gbat_str}{signal_str}{latency_str}{bitrate_str}{distance_str}");

    let image_dimensions = image.dimensions();

    let x_pos = srt_options.position.x / 100.0 * image_dimensions.0 as f32;
    let y_pos = srt_options.position.y / 100.0 * image_dimensions.1 as f32;
    let scale = srt_options.scale / 1080.0 * image_dimensions.1 as f32;

    draw_text_mut(
        image,
        Rgba([240u8, 240u8, 240u8, 10u8]),
        x_pos as i32,
        y_pos as i32,
        rusttype::Scale::uniform(scale),
        font,
        &srt_string,
    );
}

#[inline]
pub fn overlay_srt_debug_data(
    image: &mut RgbaImage,
    debug_data: &SrtDebugFrameData,
    font: &rusttype::Font,
    srt_options: &SrtOptions,
) {
    let signal_str = if srt_options.show_signal {
        format!("Signal:{}  ", debug_data.signal)
    } else {
        "".into()
    };

    let channel_str = if srt_options.show_channel {
        format!("Channel:{}  ", debug_data.channel)
    } else {
        "".into()
    };

    let latency_str = if srt_options.show_latency {
        format!("Latency:{}ms  ", debug_data.latency)
    } else {
        "".into()
    };

    let snr_str = if srt_options.show_snr {
        format!("GSNR:{:.1}  SSNR:{:.1}  ", debug_data.gsnr, debug_data.ssnr)
    } else {
        "".into()
    };

    let g_temp_str = if srt_options.show_g_temp {
        format!("G:{:.1}°C ", debug_data.gtemp)
    } else {
        "".into()
    };

    let s_temp_str = if srt_options.show_s_temp {
        format!("S:{:.1}°C  ", debug_data.stemp)
    } else {
        "".into()
    };

    let frame_str = if srt_options.show_frame {
        format!("Frame:{}  ", debug_data.frame)
    } else {
        "".into()
    };

    let err_str = if srt_options.show_err {
        format!("GErr:{}  SErr:{}  SErrExt:{}  ", debug_data.gerr, debug_data.serr, debug_data.serr_ext)
    } else {
        "".into()
    };

    let iso_str = if srt_options.show_iso {
        format!("ISO:{}  ISOMode:{}  ISOExp:{}  ", debug_data.iso, debug_data.iso_mode, debug_data.iso_exp)
    } else {
        "".into()
    };

    let gain_str = if srt_options.show_gain {
        format!("Gain:{:.1}  GainExp:{:.1}  GainLx:{}  ", debug_data.gain, debug_data.gain_exp, debug_data.gain_lx)
    } else {
        "".into()
    };

    let cct_str = if srt_options.show_cct {
        format!("CCT:{}  ", debug_data.cct)
    } else {
        "".into()
    };

    let rb_str = if srt_options.show_rb {
        format!("RB:{:.1}  RBExt:{:.1}  ", debug_data.rb, debug_data.rb_ext)
    } else {
        "".into()
    };

    let srt_string = format!("{signal_str}{channel_str}{latency_str}{snr_str}{g_temp_str}{s_temp_str}{frame_str}{err_str}{iso_str}{gain_str}{cct_str}{rb_str}");

    let image_dimensions = image.dimensions();

    let x_pos = srt_options.position.x / 100.0 * image_dimensions.0 as f32;
    let y_pos = srt_options.position.y / 100.0 * image_dimensions.1 as f32;
    let scale = srt_options.scale / 1080.0 * image_dimensions.1 as f32;

    draw_text_mut(
        image,
        Rgba([240u8, 240u8, 240u8, 10u8]),
        x_pos as i32,
        y_pos as i32,
        rusttype::Scale::uniform(scale),
        font,
        &srt_string,
    );
}
