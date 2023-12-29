use anyhow::{Context, Result};
use inquire::Confirm;

use super::{
    config::{render_config, render_config_with_skkipable},
    Prompt,
};

pub struct BooleanPrompt<'a> {
    message: &'a str,
}

impl<'a> BooleanPrompt<'a> {
    pub fn new(message: &'a str) -> Self {
        Self { message }
    }

    fn create_prompt(&self) -> Confirm {
        Confirm::new(self.message).with_placeholder("y/n")
    }
}

impl<'a> Prompt<bool> for BooleanPrompt<'a> {
    fn prompt(&self) -> Result<bool> {
        self.create_prompt()
            .with_render_config(render_config())
            .prompt()
            .with_context(|| format!("Failed to get {}", self.message))
    }

    fn prompt_skippable(&self) -> Result<Option<bool>> {
        self.create_prompt()
            .with_render_config(render_config_with_skkipable())
            .prompt_skippable()
            .with_context(|| format!("Failed to get {}", self.message))
    }
}
