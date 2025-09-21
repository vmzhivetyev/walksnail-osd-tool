use std::{
    cell::RefCell,
    collections::HashMap,
    path::PathBuf,
};

use derivative::Derivative;
use image::{imageops::FilterType, io::Reader, DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};

// Cache for resized glyphs with size limit (simplified key since size is constant per session)
#[derive(Derivative, Clone, Debug)]
pub struct ImageCache {
    cache: RefCell<HashMap<usize, ImageBuffer<Rgba<u8>, Vec<u8>>>>,
    max_size: usize,
}

impl ImageCache {
    pub fn new(max_size: usize) -> Self {
        ImageCache {
            cache: RefCell::new(HashMap::new()),
            max_size,
        }
    }

    pub fn get(&self, index: usize) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        self.cache.borrow().get(&index).cloned()
    }

    pub fn insert(&self, index: usize, image: ImageBuffer<Rgba<u8>, Vec<u8>>) {
        let mut cache = self.cache.borrow_mut();
        
        // If cache is full, don't add new entries (simple bounded cache)
        if cache.len() >= self.max_size {
            return;
        }
        
        cache.insert(index, image);
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
    cache: ImageCache,
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

        // Create cache for resized glyphs (grows as needed, bounded)
        let cache = ImageCache::new(256); // Reduced cache size to prevent memory bloat

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
    ) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        // Check cache first (simplified key since size is constant per session)
        if let Some(cached_image) = self.cache.get(index) {
            return Some(cached_image);
        }

        // Cache miss - resize from original glyph
        let final_size = Dimension {
            width: ((desired_size.width as f32) * size_class.multiplier()).round() as u32,
            height: ((desired_size.height as f32) * size_class.multiplier()).round() as u32,
        };

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

            // Cache the resized image (simplified key)
            self.cache.insert(index, resized_image.clone());
            resized_image
        })
    }
}

fn split_characters(
    font_image: &DynamicImage,
    character_size: &Dimension<u32>,
    font_type: &FontType,
) -> Vec<ImageBuffer<Rgba<u8>, Vec<u8>>> {
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
