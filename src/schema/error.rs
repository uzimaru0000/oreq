use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("Reference error: failed to resolve reference {0}")]
    ReferenceError(String),
    #[error("Unsupported external reference. Please bundle your schema")]
    UnsupportedExternalReference,
}
