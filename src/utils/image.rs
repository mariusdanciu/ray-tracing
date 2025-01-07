use std::io::Cursor;

use crate::objects::Texture;

use super::errors::AppError;
use image::ImageReader;

pub struct ImageUtils {
}

impl ImageUtils {
    
    pub fn load_image(path: impl Into<String>) -> Result<Texture, AppError> {
        let p: String = path.into();
        let img = ImageReader::open(p.clone())?.decode()?;
        let mut bytes: Vec<u8> = Vec::new();
        img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;
        Ok(Texture{
            path: p,
            width: img.width(),
            height: img.height(),
            bytes
        })
    }
}