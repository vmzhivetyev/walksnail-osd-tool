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

    fn generate_key(index: usize, size: &CharacterSize) -> u64 {
        let mut hasher = DefaultHasher::new();
        index.hash(&mut hasher);
        size.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get(&self, index: usize, size: &CharacterSize) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        let key = Self::generate_key(index, size);
        self.cache.borrow().get(&key).cloned() // Cloning the image to return a copy
    }

    pub fn insert(&self, index: usize, size: &CharacterSize, image: ImageBuffer<Rgba<u8>, Vec<u8>>) {
        let key = Self::generate_key(index, size);
        self.cache.borrow_mut().insert(key, image);
    }
}

use super::{
    dimensions::{detect_dimensions, CharacterSize, FontType},
    error::FontFileError,
};

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FontFile {
    pub file_path: PathBuf,
    pub character_count: u32,
    pub character_size: CharacterSize,
    pub font_type: FontType,
    #[derivative(Debug = "ignore")]
    characters: Vec<RgbaImage>,
    cache: ImageCache,
}

impl FontFile {
    #[tracing::instrument(ret, err)]
    pub fn open(path: PathBuf) -> Result<Self, FontFileError> {
        let font_image = Reader::open(&path)?.decode()?;
        let (width, height) = font_image.dimensions();
        let (character_size, font_type, character_count) = detect_dimensions(width, height)?;

        let characters = split_characters(&font_image, &character_size, &font_type, character_count);

        Ok(Self {
            file_path: path,
            character_count,
            character_size,
            font_type,
            characters,
            cache: ImageCache::new(),
        })
    }

    pub fn get_character(&self, index: usize, size: &CharacterSize) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        if let Some(cached_image) = self.cache.get(index, size) {
            return Some(cached_image.clone());
        }

        self.characters.get(index).map(|original_image| {
            let resized_image = if size.width() != self.character_size.width() || size.height() != self.character_size.height() {
                image::imageops::resize(original_image, size.width(), size.height(), FilterType::Lanczos3)
            } else {
                original_image.clone()
            };

            // Cache the resized image
            self.cache.insert(index, size, resized_image.clone());
            println!("Cache char {}", index);
            resized_image
        })
    }
}

fn split_characters(
    font_image: &DynamicImage,
    character_size: &CharacterSize,
    font_type: &FontType,
    character_count: u32,
) -> Vec<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let pages = font_type.pages();
    let char_width = character_size.width();
    let char_height = character_size.height();

    let mut char_vec = Vec::with_capacity((character_count * pages) as usize);

    for page_idx in 0..pages {
        let x = page_idx * char_width;
        for char_idx in 0..character_count {
            let y = char_idx * char_height;
            let char = font_image.view(x, y, char_width, char_height).to_image();
            char_vec.push(char);
        }
    }

    char_vec
}
