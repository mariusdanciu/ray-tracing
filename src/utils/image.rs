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
        let rgb8 = img.clone().into_rgb8();
        let k = rgb8.as_ref();
        let (w, h) = (img.width(), img.height());

        let  bytes: Vec<u8> = Vec::from(k);

        println!("img len {}", bytes.len());
        Ok(Texture{
            path: p,
            width: w,
            height: h,
            bytes
        })
    }
}