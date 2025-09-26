use serde::{Deserialize, Serialize};

use crate::util::Coordinates;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrtOptions {
    pub position: Coordinates<f32>,
    pub scale: f32,
    pub no_srt: bool,

    pub show_time: bool,
    pub show_sbat: bool,
    pub show_gbat: bool,
    pub show_signal: bool,
    pub show_latency: bool,
    pub show_bitrate: bool,
    pub show_distance: bool,

    // debug srt data
    pub show_channel: bool,
    pub show_gsnr: bool,
    pub show_ssnr: bool,
    pub show_gtemp: bool,
    pub show_stemp: bool,
    pub show_fps: bool,
    pub show_err: bool,
    pub show_settings_cam: bool,
    pub show_actual_cam: bool,
    pub show_cct: bool,
    pub show_rb: bool,
    pub show_sp: bool,
    pub show_gp: bool,
    pub show_stp: bool,
    pub show_gtp: bool,

    // DJI-specific fields
    pub show_flight_time: bool,
    pub show_uav_bat_cells: bool,
    pub show_gls_bat_cells: bool,
    pub show_rc_signal: bool,
}

impl Default for SrtOptions {
    fn default() -> Self {
        Self {
            position: Coordinates::new(1.5, 95.0),
            scale: 35.0,
            no_srt: false,

            show_time: false,
            show_sbat: false,
            show_gbat: false,
            show_signal: true,
            show_latency: true,
            show_bitrate: true,
            show_distance: true,

            // debug srt data
            show_channel: true,
            show_gsnr: true,
            show_ssnr: true,
            show_gtemp: false,
            show_stemp: true,
            show_fps: false,
            show_err: true,
            show_settings_cam: false,
            show_actual_cam: true,
            show_cct: false,
            show_rb: false,
            show_sp: false,
            show_gp: false,
            show_stp: false,
            show_gtp: false,

            // DJI-specific fields
            show_flight_time: true,
            show_uav_bat_cells: false,
            show_gls_bat_cells: false,
            show_rc_signal: false,
        }
    }
}
