use std::str::FromStr;

use clap::ValueEnum;

use crate::req::RequestInit;

pub(crate) mod curl;
pub(crate) mod fetch;

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("Invalid Body: {0}")]
    InvalidBody(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum FromStrError {
    #[error("No implements for {0}")]
    NoImplements(String),
}

pub(crate) trait RequestFormatter {
    fn format(&self, req: &RequestInit) -> Result<String, FormatError>;
}

#[derive(Debug, Clone, ValueEnum)]
pub(crate) enum Formatter {
    #[value(help = "Curl argument format")]
    Curl,
    #[value(help = "Fetch style for WebStandard API")]
    Fetch,
}

impl From<Formatter> for Box<dyn RequestFormatter> {
    fn from(f: Formatter) -> Self {
        match f {
            Formatter::Curl => Box::new(curl::CurlFormatter),
            Formatter::Fetch => Box::new(fetch::FetchFormatter),
        }
    }
}

impl FromStr for Formatter {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "curl" => Ok(Self::Curl),
            "fetch" => Ok(Self::Fetch),
            _ => Err(FromStrError::NoImplements(s.to_string())),
        }
    }
}
