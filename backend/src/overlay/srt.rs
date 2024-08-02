use image::{Rgba, RgbaImage};
use imageproc::{drawing::draw_text_mut};
use rusttype::{Font, Scale, point};

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

    let channel_str = if srt_options.show_channel {
        format!("Ch:{}  ", srt_data.channel)
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

    let srt_string = format!("{signal_str}{channel_str}{time_str}{gbat_str}{sbat_str}{latency_str}{bitrate_str}{distance_str}");

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
    srt_debug_data: &SrtDebugFrameData,
    font: &rusttype::Font,
    srt_options: &SrtOptions,
) {
    let mut srt_string = String::new();

    if srt_options.show_channel {
        srt_string.push_str(&format!("CH:{} ", srt_debug_data.channel));
    }
    if srt_options.show_signal {
        srt_string.push_str(&format!("MCS:{} ", srt_debug_data.signal));
    }
    if srt_options.show_gp {
        srt_string.push_str(&format!("GP[{} {} {} {}] ", srt_debug_data.gp1, srt_debug_data.gp2, srt_debug_data.gp3, srt_debug_data.gp4))
    }
    if srt_options.show_sp {
        srt_string.push_str(&format!("SP[{} {} {} {}] ", srt_debug_data.sp1, srt_debug_data.sp2, srt_debug_data.sp3, srt_debug_data.sp4))
    }
    if srt_options.show_gtp {
        srt_string.push_str(&format!("GTP:{} ", srt_debug_data.gtp))
    }
    if srt_options.show_stp {
        srt_string.push_str(&format!("STP:{} ", srt_debug_data.stp))
    }
    if srt_options.show_gsnr {
        srt_string.push_str(&format!("GSNR:{:.1} ", srt_debug_data.gsnr));
    }
    if srt_options.show_ssnr {
        srt_string.push_str(&format!("SSNR:{:.1} ", srt_debug_data.ssnr));
    }
    if srt_options.show_gtemp {
        srt_string.push_str(&format!("GTemp:{:.1}°C ", srt_debug_data.gtemp));
    }
    if srt_options.show_stemp {
        srt_string.push_str(&format!("STemp:{:.1}°C ", srt_debug_data.stemp));
    }
    if srt_options.show_latency {
        srt_string.push_str(&format!("Delay:{} ", srt_debug_data.latency));
    }
    if srt_options.show_fps {
        srt_string.push_str(&format!("FPS:{} ", srt_debug_data.fps));
    }
    if srt_options.show_err {
        srt_string.push_str(&format!("Gerr:{} SErr:{} {} ", srt_debug_data.gerr, srt_debug_data.serr, srt_debug_data.serr_ext));
    }
    if srt_options.show_settings_cam {
        srt_string.push_str(&format!("[ISO:{} Mode:{} Exp:{}] ", srt_debug_data.iso, srt_debug_data.iso_mode, srt_debug_data.iso_exp));
    }
    if srt_options.show_actual_cam {
        srt_string.push_str(&format!("[ISO_Gain:{:.2} Exp:{:.3}ms Lx:{}] ", srt_debug_data.gain, srt_debug_data.gain_exp, srt_debug_data.gain_lx));
    }
    if srt_options.show_cct {
        srt_string.push_str(&format!("[CCT:{}] ", srt_debug_data.cct));
    }
    if srt_options.show_rb {
        srt_string.push_str(&format!("[RB:{:.2} {:.2}] ", srt_debug_data.rb, srt_debug_data.rb_ext));
    }

    overlay_string(image, &srt_string, font, srt_options)
}

fn overlay_string(
    image: &mut RgbaImage,
    srt_string: &String,
    font: &rusttype::Font,
    srt_options: &SrtOptions,
) {
    let image_dimensions = image.dimensions();
    let scale = Scale::uniform(srt_options.scale / 1080.0 * image_dimensions.1 as f32);

    // Function to measure text width
    fn text_width(font: &Font, scale: Scale, text: &str) -> f32 {
        font.layout(text, scale, point(0.0, 0.0))
            .map(|g| g.unpositioned().h_metrics().advance_width)
            .sum()
    }

    let max_width = image_dimensions.0 as f32;
    let words: Vec<&str> = srt_string.split_whitespace().collect();
    let mut line1 = String::new();
    let mut line2 = String::new();
    let mut on_first_line = true;

    for word in words {
        let potential_line = if on_first_line {
            if line1.is_empty() { word.to_string() } else { format!("{} {}", line1, word) }
        } else {
            if line2.is_empty() { word.to_string() } else { format!("{} {}", line2, word) }
        };

        if text_width(font, scale, &potential_line) <= max_width {
            if on_first_line {
                line1 = potential_line;
            } else {
                line2 = potential_line;
            }
        } else if on_first_line {
            on_first_line = false;
            line2 = word.to_string();
        } else {
            break; // If we can't fit on two lines, stop adding words
        }
    }

    let x_pos = (srt_options.position.x / 100.0 * image_dimensions.0 as f32).round() as i32;
    let y_pos = (srt_options.position.y / 100.0 * image_dimensions.1 as f32).round() as i32;

    draw_text_mut(
        image,
        Rgba([240u8, 240u8, 240u8, 255u8]),
        x_pos,
        y_pos,
        scale,
        font,
        &line1,
    );

    if !line2.is_empty() {
        draw_text_mut(
            image,
            Rgba([240u8, 240u8, 240u8, 255u8]),
            x_pos,
            y_pos + scale.y as i32,
            scale,
            font,
            &line2,
        );
    }
}
