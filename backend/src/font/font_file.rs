use std::{cell::RefCell, hash::{DefaultHasher, Hash, Hasher}, path::PathBuf};

use derivative::Derivative;
use image::{imageops::FilterType, io::Reader, DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};

use std::collections::HashMap;


// Cache structure
#[derive(Derivative, Clone, Debug)]
pub struct ImageCache {
    cache: RefCell<HashMap<u64, ImageBuffer<Rgba<u8>, Vec<u8>>>>,
}

impl ImageCache {
    pub fn new() -> Self {
        ImageCache {
            cache: RefCell::new(HashMap::new()),
        }
    }

    fn generate_key(index: usize, size: &CharacterSizeClass) -> u64 {
        let mut hasher = DefaultHasher::new();
        index.hash(&mut hasher);
        size.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get(&self, index: usize, size: &CharacterSizeClass) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        let key = Self::generate_key(index, size);
        self.cache.borrow().get(&key).cloned() // Cloning the image to return a copy
    }

    pub fn insert(&self, index: usize, size: &CharacterSizeClass, image: ImageBuffer<Rgba<u8>, Vec<u8>>) {
        let key = Self::generate_key(index, size);
        self.cache.borrow_mut().insert(key, image);
    }
}

use crate::util::Dimension;

use super::{
    dimensions::{detect_font_character_size, CharacterSizeClass, FontType},
    error::FontFileError,
};

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

        Ok(Self {file_path:path,font_type,font_character_size,characters,cache:ImageCache::new(), character_count })
    }

    pub fn get_character(&self, index: usize, size_class: &CharacterSizeClass, desired_size: Dimension<u32>) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        if let Some(cached_image) = self.cache.get(index, size_class) {
            return Some(cached_image.clone());
        }

        let final_size = Dimension { 
            width: ((desired_size.width as f32) * size_class.multiplier()).round() as u32, 
            height: ((desired_size.height as f32) * size_class.multiplier()).round() as u32
        };

        // this allows us to use single color fonts for multicolor osd files.
        let wrapped_char_index = index % self.character_count as usize;

        self.characters.get(wrapped_char_index).map(|original_image| {
            let resized_image = if final_size != self.font_character_size {
                image::imageops::resize(original_image, final_size.width, final_size.height, FilterType::Lanczos3)
            } else {
                original_image.clone()
            };

            // Cache the resized image
            self.cache.insert(index, size_class, resized_image.clone());
            println!("Cache glyph image {}", index);
            resized_image
        })
    }
}

fn split_characters(
    font_image: &DynamicImage,
    character_size: &Dimension<u32>,
    font_type: &FontType
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
            let char = font_image.view(x, y, character_size.width, character_size.height).to_image();
            char_vec.push(char);
        }
    }

    char_vec
}
