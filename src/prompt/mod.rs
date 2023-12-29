use anyhow::Result;

pub mod api;
mod array;
mod boolean;
mod config;
mod integer;
mod number;
mod schema;
mod string;
mod validator;

pub trait Prompt<T> {
    fn prompt(&self) -> Result<T>;
    fn prompt_skippable(&self) -> Result<Option<T>>;
}
