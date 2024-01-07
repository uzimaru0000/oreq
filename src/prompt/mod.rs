use anyhow::Result;
use serde_json::Value;

pub mod api;
mod array;
mod boolean;
mod config;
mod number;
mod schema;
mod string;
mod validator;

pub trait Prompt {
    fn prompt(&self) -> Result<Value>;
    fn prompt_skippable(&self) -> Result<Option<Value>>;
}
