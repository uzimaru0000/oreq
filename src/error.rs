#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Invalid schema")]
    SchemaError,
    #[error("No servers in schema")]
    NoServers,
    #[error(transparent)]
    PromptError(#[from] inquire::error::InquireError),
    #[error("Failed to parse URL")]
    ParseError(#[from] url::ParseError),
}

impl AppError {
    pub fn show(&self) -> (String, i32) {
        match self {
            AppError::PromptError(e) => match e {
                inquire::InquireError::OperationCanceled => ("Operation canceled".to_string(), 0),
                inquire::InquireError::OperationInterrupted => {
                    ("Operation interrupted".to_string(), 0)
                }
                _ => (format!("Error: {}", e), 1),
            },
            _ => (format!("Error: {}", self), 1),
        }
    }
}
