use serde::{Deserialize, Serialize};

use crate::util::Coordinates;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrtOptions {
    pub position: Coordinates<f32>,
    pub scale: f32,
    pub show_time: bool,
    pub show_sbat: bool,
    pub show_gbat: bool,
    pub show_signal: bool,
    pub show_latency: bool,
    pub show_bitrate: bool,
    pub show_distance: bool,
    
    // debug srt data
    pub show_channel: bool,
    pub show_snr: bool,
    pub show_g_temp: bool,
    pub show_s_temp: bool,
    pub show_frame: bool,
    pub show_err: bool,
    pub show_iso: bool,
    pub show_gain: bool,
    pub show_cct: bool,
    pub show_rb: bool,
}

impl Default for SrtOptions {
    fn default() -> Self {
        Self {
            position: Coordinates::new(1.5, 95.0),
            scale: 35.0,
            show_time: false,
            show_sbat: false,
            show_gbat: false,
            show_signal: true,
            show_latency: true,
            show_bitrate: true,
            show_distance: true,

            // debug srt data
            show_channel: true,
            show_snr: true,
            show_g_temp: true,
            show_s_temp: true,
            show_frame: true,
            show_err: true,
            show_iso: true,
            show_gain: true,
            show_cct: true,
            show_rb: true,
        }
    }
}
