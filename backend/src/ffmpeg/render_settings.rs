use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenderSettings {
    pub selected_encoder_idx: usize,
    pub show_undetected_encoders: bool,
    pub bitrate_mbps: u32,
    pub keep_quality: bool,
    pub upscale: bool,
    pub rescale_to_4x3_aspect: bool,
    pub rendering_live_view: bool,
    pub use_chroma_key: bool,
    pub chroma_key: [f32; 4],
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            selected_encoder_idx: 0,
            show_undetected_encoders: false,
            bitrate_mbps: 40,
            keep_quality: true,
            upscale: false,
            rescale_to_4x3_aspect: false,
            rendering_live_view: true,
            use_chroma_key: false,
            chroma_key: [1.0 / 255.0, 177.0 / 255.0, 64.0 / 255.0, 1.0],
        }
    }
}
