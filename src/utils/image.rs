use std::io::Cursor;

use super::errors::AppError;
use image::ImageReader;

pub struct ImageUtils {
}

impl ImageUtils {
    
    pub fn load_image(path: impl Into<String>) -> Result<Vec<u8>, AppError> {
        let img = ImageReader::open(path.into())?.decode()?;
        let mut bytes: Vec<u8> = Vec::new();
        img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;
        Ok(bytes)
    }
}