use std::collections::HashSet;

use derivative::Derivative;
use serde::{Deserialize, Serialize};

use crate::{font::CharacterSizeClass, util::{Coordinates, Dimension}};

#[derive(Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default, Debug)]
pub struct OsdOptions {
    pub position: Coordinates<i32>,
    #[derivative(Default(value = "false"))]
    pub adjust_playback_speed: bool,
    pub no_osd: bool,
    #[derivative(Default(value = "1.0"))]
    #[serde(skip)]
    pub osd_playback_speed_factor: f32,
    pub masked_grid_positions: HashSet<Coordinates<u32>>,
    #[derivative(Default(value = "0.0"))]
    #[serde(skip)]
    pub osd_playback_offset: f32,
    #[serde(skip)]
    pub character_size_class: Option<CharacterSizeClass>,
    #[serde(skip)]
    pub character_size: Dimension<u32>,
}

impl OsdOptions {
    pub fn get_mask(&self, position: &Coordinates<u32>) -> bool {
        self.masked_grid_positions.contains(position)
    }

    pub fn toggle_mask(&mut self, position: Coordinates<u32>) {
        if self.masked_grid_positions.contains(&position) {
            self.masked_grid_positions.remove(&position);
        } else {
            self.masked_grid_positions.insert(position);
        }
    }

    pub fn reset_mask(&mut self) {
        self.masked_grid_positions.clear();
    }
}
