use parse_display::FromStr;
use std::str::FromStr;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct SrtFrame {
    pub start_time_secs: f32,
    pub end_time_secs: f32,
    pub data: Option<SrtFrameData>,
    pub debug_data: Option<SrtDebugFrameData>,
}

#[derive(Debug, FromStr, Clone, PartialEq)]
#[display("Signal:{signal} CH:{channel} FlightTime:{flight_time} SBat:{sky_bat}V GBat:{ground_bat}V Delay:{latency}ms Bitrate:{bitrate_mbps}Mbps Distance:{distance}m")]
pub struct SrtFrameData {
    pub signal: u8,
    pub channel: u8,
    pub flight_time: u32,
    pub sky_bat: f32,
    pub ground_bat: f32,
    pub latency: u32,
    pub bitrate_mbps: f32,
    pub distance: u32,
}

// See https://walksnail.wiki/en/Debug
#[derive(Debug, Clone, PartialEq)]
pub struct SrtDebugFrameData {
    pub signal: i8,
    pub channel: i8,
    //pub flight_time: u32,
    //pub sky_bat: f32,
    //pub ground_bat: f32,
    pub latency: i32,
    //pub bitrate_mbps: f32,
    //pub distance: u32,
    pub sp1: i16,
    pub sp2: i16,
    pub sp3: i16,
    pub sp4: i16,
    pub gp1: i16,
    pub gp2: i16,
    pub gp3: i16,
    pub gp4: i16,
    pub gtp: i16,
    pub gtp0: i16,
    pub stp: i16,
    pub stp0: i16,
    pub gsnr: f32,
    pub ssnr: f32,
    pub gtemp: f32,
    pub stemp: f32,
    pub fps: i16,
    pub gerr: i16,
    pub serr: i16,
    pub serr_ext: i16,
    pub iso: i32,
    pub iso_mode: String,
    pub iso_exp: i32,
    pub gain: f32,
    pub gain_exp: f32,
    pub gain_lx: i16,
    pub cct: i16,
    pub rb: f32,
    pub rb_ext: f32,
}

impl FromStr for SrtDebugFrameData {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static::lazy_static! {
            static ref RE_CHANNEL_SIGNAL: Regex = Regex::new(r"CH:\s*(\d+)\s*MCS:\s*(\d+)").unwrap();
            static ref RE_SP: Regex = Regex::new(r"SP\[\s*(\d+)\s*(\d+)\s*(\d+)\s*(\d+)\s*\]").unwrap();
            static ref RE_GP: Regex = Regex::new(r"GP\[\s*(\d+)\s*(\d+)\s*(\d+)\s*(\d+)\s*\]").unwrap();
            static ref RE_GTP_STP: Regex = Regex::new(r"GTP:\s*(\d+)\s*GTP0:\s*(\d+)\s*STP:\s*(\d+)\s*STP0:\s*([-]?\d+)").unwrap();
            static ref RE_SNR_TEMP: Regex = Regex::new(r"GSNR:\s*(-?[\d.]+)\s*SSNR:\s*(-?[\d.]+)\s*Gtemp:\s*(-?[\d.]+)\s*Stemp:\s*(-?[\d.]+)").unwrap();
            static ref RE_MISC: Regex = Regex::new(r"Delay:\s*(\d+)ms\s*Frame:\s*(\d+)\s*Gerr:\s*(\d+)\s*SErr:\s*(\d+)\s*(\d+)").unwrap();
            static ref RE_ISO: Regex = Regex::new(r"\[iso:(\d+),mode=(\w+),\s*exp:(\d+)\]").unwrap();
            static ref RE_GAIN: Regex = Regex::new(r"\[gain:([\d.]+)\s*exp:([\d.]+)ms,\s*Lx:(\d+)\]").unwrap();
            static ref RE_CCT_RB: Regex = Regex::new(r"\[cct:(\d+),\s*rb:([\d.]+)\s*([\d.]+)\]").unwrap();
        }

        let channel_signal = RE_CHANNEL_SIGNAL.captures(s).ok_or("Failed to match channel and signal")?;
        let sp = RE_SP.captures(s).ok_or("Failed to match SP values")?;
        let gp = RE_GP.captures(s).ok_or("Failed to match GP values")?;
        let gtp_stp = RE_GTP_STP.captures(s).ok_or("Failed to match GTP and STP values")?;
        let snr_temp = RE_SNR_TEMP.captures(s).ok_or("Failed to match SNR and temperature values")?;
        let misc = RE_MISC.captures(s).ok_or("Failed to match miscellaneous values")?;
        let iso = RE_ISO.captures(s).ok_or("Failed to match ISO values")?;
        let gain = RE_GAIN.captures(s).ok_or("Failed to match gain values")?;
        let cct_rb = RE_CCT_RB.captures(s).ok_or("Failed to match CCT and RB values")?;

        Ok(SrtDebugFrameData {
            channel: channel_signal[1].parse().map_err(|_| "Invalid channel")?,
            signal: channel_signal[2].parse().map_err(|_| "Invalid signal")?,
            sp1: sp[1].parse().map_err(|_| "Invalid sp1")?,
            sp2: sp[2].parse().map_err(|_| "Invalid sp2")?,
            sp3: sp[3].parse().map_err(|_| "Invalid sp3")?,
            sp4: sp[4].parse().map_err(|_| "Invalid sp4")?,
            gp1: gp[1].parse().map_err(|_| "Invalid gp1")?,
            gp2: gp[2].parse().map_err(|_| "Invalid gp2")?,
            gp3: gp[3].parse().map_err(|_| "Invalid gp3")?,
            gp4: gp[4].parse().map_err(|_| "Invalid gp4")?,
            gtp: gtp_stp[1].parse().map_err(|_| "Invalid gtp")?,
            gtp0: gtp_stp[2].parse().map_err(|_| "Invalid gtp0")?,
            stp: gtp_stp[3].parse().map_err(|_| "Invalid stp")?,
            stp0: gtp_stp[4].parse().map_err(|_| "Invalid stp0")?,
            gsnr: snr_temp[1].parse().map_err(|_| "Invalid gsnr")?,
            ssnr: snr_temp[2].parse().map_err(|_| "Invalid ssnr")?,
            gtemp: snr_temp[3].parse().map_err(|_| "Invalid gtemp")?,
            stemp: snr_temp[4].parse().map_err(|_| "Invalid stemp")?,
            latency: misc[1].parse().map_err(|_| "Invalid latency")?,
            fps: misc[2].parse().map_err(|_| "Invalid frame")?,
            gerr: misc[3].parse().map_err(|_| "Invalid gerr")?,
            serr: misc[4].parse().map_err(|_| "Invalid serr")?,
            serr_ext: misc[5].parse().map_err(|_| "Invalid serr_ext")?,
            iso: iso[1].parse().map_err(|_| "Invalid iso")?,
            iso_mode: iso[2].to_string(),
            iso_exp: iso[3].parse().map_err(|_| "Invalid iso_exp")?,
            gain: gain[1].parse().map_err(|_| "Invalid gain")?,
            gain_exp: gain[2].parse().map_err(|_| "Invalid gain_exp")?,
            gain_lx: gain[3].parse().map_err(|_| "Invalid gain_lx")?,
            cct: cct_rb[1].parse().map_err(|_| "Invalid cct")?,
            rb: cct_rb[2].parse().map_err(|_| "Invalid rb")?,
            rb_ext: cct_rb[3].parse().map_err(|_| "Invalid rb_ext")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pre_v31_36_8_srt_frame_data() {
        let line = "Signal:4 CH:8 FlightTime:0 SBat:4.7V GBat:7.2V Delay:32ms Bitrate:25Mbps Distance:7m";
        let parsed = line.parse::<SrtFrameData>();
        assert_eq!(
            parsed.expect("Failed to parse SRT frame data"),
            SrtFrameData {
                signal: 4,
                channel: 8,
                flight_time: 0,
                sky_bat: 4.7,
                ground_bat: 7.2,
                latency: 32,
                bitrate_mbps: 25.0,
                distance: 7
            }
        )
    }

    #[test]
    fn parse_v32_37_10_srt_frame_data() {
        let line = "Signal:4 CH:7 FlightTime:0 SBat:16.7V GBat:12.5V Delay:25ms Bitrate:25.0Mbps Distance:1m";
        let parsed = line.parse::<SrtFrameData>();
        assert_eq!(
            parsed.expect("Failed to parse SRT frame data"),
            SrtFrameData {
                signal: 4,
                channel: 7,
                flight_time: 0,
                sky_bat: 16.7,
                ground_bat: 12.5,
                latency: 25,
                bitrate_mbps: 25.0,
                distance: 1
            }
        )
    }

    #[test]
    fn parse_v37_42_3_debug_src_frame_data() {
        let line = "CH:4 MCS:4 SP[ 74 152 152 152] GP[ 59  65  53  60] GTP:10 GTP0:00 STP:09 STP0:-1 GSNR:21.4 SSNR:21.6 Gtemp:35 Stemp:56 Delay:35ms Frame:60  Gerr:0 SErr:0 24, [iso:0,mode=max, exp:0] [gain:0.00 exp:0.000ms, Lx:0] [cct:0, rb:0.000 0.000]";
        let parsed = line.parse::<SrtDebugFrameData>();
        assert_eq!(
            parsed.expect("Failed to parse SRT frame data"),
            SrtDebugFrameData { signal: 4, channel: 4, latency: 35, sp1: 74, sp2: 152, sp3: 152, sp4: 152, gp1: 59, gp2: 65, gp3: 53, gp4: 60, gtp: 10, gtp0: 0, stp: 9, stp0: -1, gsnr: 21.4, ssnr: 21.6, gtemp: 35.0, stemp: 56.0, fps: 60, gerr: 0, serr: 0, serr_ext: 24, iso: 0, iso_mode: "max".to_string(), iso_exp: 0, gain: 0.0, gain_exp: 0.0, gain_lx: 0, cct: 0, rb: 0.0, rb_ext: 0.0 }
        )
    }
}
