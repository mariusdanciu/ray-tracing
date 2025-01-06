use image::ImageError;

#[derive(Debug, Clone)]
pub enum AppError {
    ErrorIo,
    ErrorLoadTexture
}

impl From<std::io::Error> for AppError {
    fn from(_value: std::io::Error) -> Self {
        AppError::ErrorIo
    }
}

impl From<ImageError> for AppError {
    fn from(_value: ImageError) -> Self {
        AppError::ErrorLoadTexture
    }
}