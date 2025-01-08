use image::ImageError;

#[derive(Debug, Clone)]
pub enum AppError {
    ErrorIo(String),
    ErrorLoadTexture(String),
    ErrorString(String)
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::ErrorIo(format!("{}", value))
    }
}

impl From<ImageError> for AppError {
    fn from(value: ImageError) -> Self {
        AppError::ErrorLoadTexture(value.to_string())
    }
}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        AppError::ErrorString(value)
    }
}