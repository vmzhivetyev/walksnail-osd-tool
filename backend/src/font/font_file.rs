use std::{cell::RefCell, collections::HashMap, path::PathBuf, sync::Arc};

use derivative::Derivative;
use image::{imageops::FilterType, io::Reader, DynamicImage, GenericImageView, RgbaImage};

/// Resized glyphs, keyed by glyph index and final size, so a change of character size class or
/// video resolution never returns a glyph of the wrong size.
#[derive(Clone, Debug, Default)]
struct GlyphCache {
    cache: RefCell<HashMap<(usize, u32, u32), Arc<RgbaImage>>>,
}

impl GlyphCache {
    fn get(&self, index: usize, size: &Dimension<u32>) -> Option<Arc<RgbaImage>> {
        self.cache.borrow().get(&(index, size.width, size.height)).cloned()
    }

    fn insert(&self, index: usize, size: &Dimension<u32>, image: Arc<RgbaImage>) {
        self.cache.borrow_mut().insert((index, size.width, size.height), image);
    }
}

use super::{
    dimensions::{detect_font_character_size, CharacterSizeClass, FontType},
    error::FontFileError,
};
use crate::util::Dimension;

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FontFile {
    pub file_path: PathBuf,
    pub character_count: u32,
    pub font_type: FontType,
    pub font_character_size: Dimension<u32>,
    #[derivative(Debug = "ignore")]
    characters: Vec<RgbaImage>,
    cache: GlyphCache,
}

impl FontFile {
    #[tracing::instrument(ret, err)]
    pub fn open(path: PathBuf) -> Result<Self, FontFileError> {
        let font_image = Reader::open(&path)?.decode()?;
        let (font_file_width, font_file_height) = font_image.dimensions();
        let font_file_dimensions = Dimension::new(font_file_width, font_file_height);
        let (font_character_size, font_type) = detect_font_character_size(font_file_dimensions)?;

        let characters = split_characters(&font_image, &font_character_size, &font_type);
        let character_count = characters.len() as u32;

        let cache = GlyphCache::default();

        Ok(Self {
            file_path: path,
            font_type,
            font_character_size,
            characters,
            cache,
            character_count,
        })
    }

    pub fn get_character(
        &self,
        index: usize,
        size_class: &CharacterSizeClass,
        desired_size: Dimension<u32>,
    ) -> Option<Arc<RgbaImage>> {
        let final_size = Dimension {
            width: ((desired_size.width as f32) * size_class.multiplier()).round() as u32,
            height: ((desired_size.height as f32) * size_class.multiplier()).round() as u32,
        };

        if let Some(cached_image) = self.cache.get(index, &final_size) {
            return Some(cached_image);
        }

        // this allows us to use single color fonts for multicolor osd files.
        let wrapped_char_index = index % self.character_count as usize;

        self.characters.get(wrapped_char_index).map(|original_image| {
            let resized_image = if final_size != self.font_character_size {
                image::imageops::resize(
                    original_image,
                    final_size.width,
                    final_size.height,
                    FilterType::Triangle,
                )
            } else {
                original_image.clone()
            };

            let resized_image = Arc::new(resized_image);
            self.cache.insert(index, &final_size, resized_image.clone());
            resized_image
        })
    }
}

fn split_characters(
    font_image: &DynamicImage,
    character_size: &Dimension<u32>,
    font_type: &FontType,
) -> Vec<RgbaImage> {
    let columns: u32 = font_type.raw_value();
    let (width, height) = font_image.dimensions();
    let font_file_dimensions = Dimension::new(width, height);
    let vertical_char_count = font_file_dimensions.height / character_size.height;
    let total_chars = vertical_char_count * columns;

    let mut char_vec = Vec::with_capacity(total_chars as usize);

    for page_idx in 0..columns {
        let x = page_idx * character_size.width;
        for char_idx in 0..vertical_char_count {
            let y = char_idx * character_size.height;
            let char = font_image
                .view(x, y, character_size.width, character_size.height)
                .to_image();
            char_vec.push(char);
        }
    }

    char_vec
}
