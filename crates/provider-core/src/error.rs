use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("authentication error: {0}")]
    Auth(String),
    #[error("transport error: {0}")]
    Transport(String),
    #[error("unexpected error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, ProviderError>;
