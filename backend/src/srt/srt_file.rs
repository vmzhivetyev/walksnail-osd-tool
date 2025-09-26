use std::{path::PathBuf, time::Duration};

use derivative::Derivative;

use crate::srt::SrtFrameDataRef;

use super::{
    error::SrtFileError,
    frame::{SrtDebugFrameData, SrtFrame, SrtData, DJISrtFrameData},
    SrtFrameData,
};


#[derive(Derivative)]
#[derivative(Debug)]
pub struct SrtFile {
    pub file_path: PathBuf,
    pub has_distance: bool,
    pub duration: Duration,
    #[derivative(Debug = "ignore")]
    pub frames: Vec<SrtFrame>,
    pub data: SrtData,
}

impl SrtFile {
    #[tracing::instrument(ret, err)]
    pub fn open(path: PathBuf) -> Result<Self, SrtFileError> {
        let mut has_distance = false;
        let parsed_frames = srtparse::from_file(&path)?;
        
        if parsed_frames.is_empty() {
            return Err(SrtFileError::EmptySrtFile { } );
        }

        // Try to determine the SRT type from the first frame
        let first_text = &parsed_frames[0].text;
        
        if first_text.parse::<DJISrtFrameData>().is_ok() {
            // Parse as DJI data
            let dji_data = parsed_frames
                .iter()
                .map(|i| i.text.parse::<DJISrtFrameData>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| SrtFileError::FailedToParseData { source: e })?;
            
            let frames: Vec<SrtFrame> = parsed_frames
                .iter()
                .map(|i| SrtFrame {
                    start_time_secs: i.start_time.into_duration().as_secs_f32(),
                    end_time_secs: i.end_time.into_duration().as_secs_f32(),
                })
                .collect();

            let duration = Duration::from_secs_f32(frames.last().unwrap().end_time_secs);

            Ok(Self {
                file_path: path,
                has_distance: false, // DJI doesn't seem to have distance
                duration,
                frames,
                data: SrtData::Dji(dji_data),
            })
        } else if first_text.parse::<SrtDebugFrameData>().is_ok() {
            // Parse as Debug data
            let debug_data = parsed_frames
                .iter()
                .map(|i| i.text.parse::<SrtDebugFrameData>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| SrtFileError::ParseError { message: e })?;
            
            let frames: Vec<SrtFrame> = parsed_frames
                .iter()
                .map(|i| SrtFrame {
                    start_time_secs: i.start_time.into_duration().as_secs_f32(),
                    end_time_secs: i.end_time.into_duration().as_secs_f32(),
                })
                .collect();

            let duration = Duration::from_secs_f32(frames.last().unwrap().end_time_secs);

            Ok(Self {
                file_path: path,
                has_distance: false, // Debug format doesn't have distance
                duration,
                frames,
                data: SrtData::Debug(debug_data),
            })
        } else if first_text.parse::<SrtFrameData>().is_ok() {
            // Parse as Normal data
            let normal_data = parsed_frames
                .iter()
                .map(|i| i.text.parse::<SrtFrameData>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| SrtFileError::FailedToParseData { source: e })?;
            
            has_distance = normal_data.iter().any(|data| data.distance > 0);
            
            let frames: Vec<SrtFrame> = parsed_frames
                .iter()
                .map(|i| SrtFrame {
                    start_time_secs: i.start_time.into_duration().as_secs_f32(),
                    end_time_secs: i.end_time.into_duration().as_secs_f32(),
                })
                .collect();

            let duration = Duration::from_secs_f32(frames.last().unwrap().end_time_secs);

            Ok(Self {
                file_path: path,
                has_distance,
                duration,
                frames,
                data: SrtData::Normal(normal_data),
            })
        } else {
            Err(SrtFileError::ParseError { message: format!("Unable to parse SRT data: {}", first_text) })
        }
    }
}

impl SrtData {
    /// Get the data for a specific frame index
    pub fn get_frame_data(&self, index: usize) -> Option<SrtFrameDataRef> {
        match self {
            SrtData::Normal(data) => data.get(index).map(SrtFrameDataRef::Normal),
            SrtData::Debug(data) => data.get(index).map(SrtFrameDataRef::Debug),
            SrtData::Dji(data) => data.get(index).map(SrtFrameDataRef::Dji),
        }
    }
    
    /// Get the total number of data frames
    pub fn len(&self) -> usize {
        match self {
            SrtData::Normal(data) => data.len(),
            SrtData::Debug(data) => data.len(),
            SrtData::Dji(data) => data.len(),
        }
    }
}