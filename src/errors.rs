use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacherchError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Tantivy error: {0}")]
    Tantivy(#[from] tantivy::TantivyError),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Unsupported file extension: {0}")]
    UnsupportedExtension(String),

    #[error("PDF extraction failed for: {0}")]
    PdfExtraction(String),
}
