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
        println!("{:?}", img.color());
        let (w, h) = (img.width(), img.height());

        let  bytes: Vec<u8> = img.into_bytes();

        println!("img len {}", bytes.len());
        Ok(Texture{
            path: p,
            width: w,
            height: h,
            bytes
        })
    }
}