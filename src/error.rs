#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Invalid schema")]
    SchemaError,
    #[error("No servers in schema")]
    NoServers,
    #[error(transparent)]
    PromptError(#[from] anyhow::Error),
    #[error("Failed to parse URL")]
    ParseError(#[from] url::ParseError),
}

impl AppError {
    pub fn show(&self) -> (String, i32) {
        match self {
            AppError::PromptError(e) => (format!("Error: {}", e), 1),
            _ => (format!("Error: {}", self), 1),
        }
    }
}
