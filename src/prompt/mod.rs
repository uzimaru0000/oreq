use inquire::error::InquireResult;
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
    fn prompt(&self) -> InquireResult<Value>;
    fn prompt_skippable(&self) -> InquireResult<Option<Value>>;
}
