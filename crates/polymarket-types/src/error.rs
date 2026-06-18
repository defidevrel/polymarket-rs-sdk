use thiserror::Error;

/// Base error type for SDK failures.
#[derive(Debug, Error)]
pub enum PolymarketError {
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),
}

/// Input or schema validation failure.
#[derive(Debug, Error)]
#[error("{message}")]
pub struct ValidationError {
    pub message: String,
}

impl ValidationError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}
