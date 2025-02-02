use std::fmt::Display;

use derivative::Derivative;
use serde::{Deserialize, Serialize};

use super::FontFileError;
use crate::util::Dimension;

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize, Derivative)]
pub enum CharacterSizeClass {
    XSmall,
    Small,
    Normal,
    Large,
    XLarge,
}

impl CharacterSizeClass {
    pub const fn multiplier(&self) -> f32 {
        match self {
            CharacterSizeClass::XSmall => 0.6,
            CharacterSizeClass::Small => 0.8,
            CharacterSizeClass::Normal => 1.0,
            CharacterSizeClass::Large => 1.1,
            CharacterSizeClass::XLarge => 1.2,
        }
    }
}

impl Display for CharacterSizeClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CharacterSizeClass::XSmall => "XS",
                CharacterSizeClass::Small => "S",
                CharacterSizeClass::Normal => "Normal",
                CharacterSizeClass::Large => "L",
                CharacterSizeClass::XLarge => "XL",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontType {
    Standard = 1,
    TwoPages = 2,
    ThreePages = 3,
    FourColor = 4,
}

impl FontType {
    pub fn from_raw_value(raw: u32) -> Option<Self> {
        match raw {
            1 => Some(FontType::Standard),
            2 => Some(FontType::TwoPages),
            3 => Some(FontType::ThreePages),
            4 => Some(FontType::FourColor),
            _ => None,
        }
    }

    pub fn raw_value(&self) -> u32 {
        *self as u32
    }
}

pub fn detect_font_character_size(font_file_size: Dimension<u32>) -> Result<(Dimension<u32>, FontType), FontFileError> {
    // This will never change neither in Betaflight nor in iNAV so we just hardcode it.
    let vertical_characters_count = 256;

    if font_file_size.height % vertical_characters_count != 0 {
        return Err(FontFileError::InvalidFontFileHeight {
            height: font_file_size.height,
        });
    }

    let single_char_height = font_file_size.height / vertical_characters_count;

    // Fonts are all done in 2:3 aspect ratio of a single char:
    // * 72x108
    // * 24x36
    // * 36x54
    let single_char_width = single_char_height * 2 / 3;

    if font_file_size.width % single_char_width != 0 {
        return Err(FontFileError::InvalidFontFileWidth {
            width: font_file_size.width,
        });
    }

    // aka columns in the font file
    let number_of_colors = font_file_size.width / single_char_width;

    Ok((
        Dimension {
            width: single_char_width,
            height: single_char_height,
        },
        FontType::from_raw_value(number_of_colors).unwrap_or(FontType::Standard),
    ))
}
