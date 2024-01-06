use anyhow::{Context, Result};
use inquire::Confirm;
use serde_json::{json, Value};

use super::{
    config::{render_config, render_config_with_skkipable},
    Prompt,
};

pub struct BooleanPrompt<'a> {
    message: &'a str,
    description: Option<&'a str>,
}

impl<'a> BooleanPrompt<'a> {
    pub fn new(message: &'a str, description: Option<&'a str>) -> Self {
        Self {
            message,
            description,
        }
    }

    fn create_prompt(&self) -> Confirm {
        let mut prompt = Confirm::new(self.message).with_placeholder("y/n");
        prompt.help_message = self.description;

        prompt
    }
}

impl<'a> Prompt for BooleanPrompt<'a> {
    fn prompt(&self) -> Result<Value> {
        self.create_prompt()
            .with_render_config(render_config())
            .prompt()
            .map(|x| json!(x))
            .with_context(|| format!("Failed to get {}", self.message))
    }

    fn prompt_skippable(&self) -> Result<Option<Value>> {
        self.create_prompt()
            .with_render_config(render_config_with_skkipable())
            .prompt_skippable()
            .map(|x| x.map(|x| json!(x)))
            .with_context(|| format!("Failed to get {}", self.message))
    }
}
