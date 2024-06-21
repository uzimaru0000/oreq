use oreq::schema::error::SchemaError;

use crate::fmt::FormatError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Invalid schema")]
    SchemaParseError,
    #[error("No servers in schema")]
    NoServers,
    #[error(transparent)]
    SchemaError(#[from] SchemaError),
    #[error(transparent)]
    PromptError(#[from] promptuity::Error),
    #[error(transparent)]
    AnyError(#[from] anyhow::Error),
    #[error("Failed to parse URL")]
    ParseError(#[from] url::ParseError),
    #[error("transparent")]
    FormatError(#[from] FormatError),
}

impl AppError {
    pub fn show(&self) -> (String, i32) {
        match self {
            AppError::PromptError(e) => match e {
                promptuity::Error::Cancel => ("Prompt cancelled".to_string(), 0),
                _ => (format!("Error: {}", e), 1),
            },
            _ => (format!("Error: {}", self), 1),
        }
    }
}
