use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to create temporary file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),
}
