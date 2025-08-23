use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransformerError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    #[error("Unsupported provider: {0}")]
    UnsupportedProvider(String),
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Tool conversion error: {0}")]
    ToolConversion(String),
    #[error("Message conversion error: {0}")]
    MessageConversion(String),
    #[error("Provider error: {0}")]
    ProviderError(String),
    #[error("Configuration error: {0}")]
    Configuration(String),
}

pub type TransformerResult<T> = Result<T, TransformerError>;