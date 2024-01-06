use anyhow::{Context, Result};
use inquire::{CustomType, Select};
use openapiv3::NumberType;
use serde_json::{json, Value};

use crate::prompt::validator::RangeValidator;

use super::{
    config::{render_config, render_config_with_skkipable},
    Prompt,
};

pub struct NumberPrompt<'a> {
    message: &'a str,
    description: Option<&'a str>,
    number: &'a NumberType,
}

impl<'a> NumberPrompt<'a> {
    pub fn new(message: &'a str, description: Option<&'a str>, number: &'a NumberType) -> Self {
        Self {
            message,
            description,
            number,
        }
    }

    fn create_select_prompt(&self) -> Option<Select<Value>> {
        let enumeration = self
            .number
            .enumeration
            .iter()
            .map(|x| x.map(|x| json!(x)))
            .filter_map(|x| x.to_owned())
            .collect::<Vec<_>>();

        if !enumeration.is_empty() {
            let mut prompt = Select::new(self.message, enumeration);
            prompt.help_message = self.description;

            Some(prompt)
        } else {
            None
        }
    }

    fn create_prompt(&self) -> CustomType<Value> {
        let mut prompt = CustomType::new(self.message)
            .with_parser(&|x| Ok(json!(x)))
            .with_validator(RangeValidator {
                min: self.number.minimum,
                max: self.number.maximum,
            });
        prompt.help_message = self.description;

        prompt
    }
}

impl<'a> Prompt for NumberPrompt<'a> {
    fn prompt(&self) -> Result<Value> {
        let select = self.create_select_prompt();

        if let Some(select) = select {
            select.with_render_config(render_config()).prompt()
        } else {
            self.create_prompt()
                .with_render_config(render_config())
                .prompt()
        }
        .with_context(|| format!("Failed to get {}", self.message))
    }

    fn prompt_skippable(&self) -> Result<Option<Value>> {
        let select = self.create_select_prompt();

        if let Some(select) = select {
            select
                .with_render_config(render_config_with_skkipable())
                .prompt_skippable()
        } else {
            self.create_prompt()
                .with_render_config(render_config_with_skkipable())
                .prompt_skippable()
        }
        .with_context(|| format!("Failed to get {}", self.message))
    }
}
