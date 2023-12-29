use anyhow::{anyhow, Context, Result};
use inquire::{required, Select, Text};
use openapiv3::StringType;

use crate::prompt::validator::RangeValidator;

use super::{
    config::{render_config, render_config_with_skkipable},
    Prompt,
};

pub struct StringPrompt<'a> {
    message: &'a str,
    string: &'a StringType,
    is_required: bool,
}

impl<'a> StringPrompt<'a> {
    pub fn new(message: &'a str, string: &'a StringType, is_required: bool) -> Self {
        Self {
            message,
            string,
            is_required,
        }
    }

    fn create_select_prompt(&self) -> Option<Select<String>> {
        let enumeration = self
            .string
            .enumeration
            .iter()
            .filter_map(|x| x.to_owned())
            .collect::<Vec<_>>();

        if !enumeration.is_empty() {
            Some(Select::new(self.message, enumeration))
        } else {
            None
        }
    }

    fn create_text_prompt(&self) -> Text {
        let prompt = Text::new(self.message).with_validator(RangeValidator {
            min: self.string.min_length,
            max: self.string.max_length,
        });

        if self.is_required {
            prompt.with_validator(required!())
        } else {
            prompt
        }
    }
}

impl<'a> Prompt<String> for StringPrompt<'a> {
    fn prompt(&self) -> Result<String> {
        let select = self.create_select_prompt();

        if let Some(select) = select {
            return select
                .with_render_config(render_config())
                .prompt()
                .with_context(|| format!("Failed to get {}", self.message));
        };

        self.create_text_prompt()
            .with_render_config(render_config())
            .prompt()
            .with_context(|| format!("Failed to get {}", self.message))
    }

    fn prompt_skippable(&self) -> Result<Option<String>> {
        let select = self.create_select_prompt();

        if let Some(select) = select {
            return select
                .with_render_config(render_config_with_skkipable())
                .prompt_skippable()
                .with_context(|| anyhow!("Failed to get {}", self.message));
        };

        self.create_text_prompt()
            .with_render_config(render_config_with_skkipable())
            .prompt_skippable()
            .with_context(|| anyhow!("Failed to get {}", self.message))
    }
}
