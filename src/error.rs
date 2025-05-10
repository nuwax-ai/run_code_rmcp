use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to execute code: {0}")]
    ExecutionError(String),
    
    #[error("Failed to parse output: {0}")]
    ParseError(String),
    
    #[error("Failed to create temporary file: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Failed to parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    
    #[error("Handler function not found")]
    HandlerNotFound,
    
    #[error("MCP error: {0}")]
    McpError(String),
    
    #[error("{0}")]
    Other(String),
}

pub type AppResult<T> = Result<T, AppError>; 