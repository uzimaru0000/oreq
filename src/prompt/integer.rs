use anyhow::{Context, Result};
use inquire::{CustomType, Select};
use openapiv3::IntegerType;
use serde_json::{json, Value};

use crate::prompt::validator::RangeValidator;

use super::{
    config::{render_config, render_config_with_skkipable},
    Prompt,
};

pub struct IntegerPrompt<'a> {
    message: &'a str,
    description: Option<&'a str>,
    integer: &'a IntegerType,
}

impl<'a> IntegerPrompt<'a> {
    pub fn new(message: &'a str, description: Option<&'a str>, integer: &'a IntegerType) -> Self {
        Self {
            message,
            description,
            integer,
        }
    }

    fn create_select_prompt(&self) -> Option<Select<Value>> {
        let enumeration = self
            .integer
            .enumeration
            .iter()
            .filter_map(|x| x.map(|x| json!(x)).to_owned())
            .collect::<Vec<_>>();

        if !enumeration.is_empty() {
            let mut prompt = Select::new(self.message, enumeration);
            prompt.help_message = self.description;

            Some(prompt)
        } else {
            None
        }
    }

    fn create_text_prompt(&self) -> CustomType<Value> {
        let mut prompt = CustomType::new(self.message)
            .with_parser(&|x| Ok(json!(x)))
            .with_validator(RangeValidator {
                min: self.integer.minimum.map(|x| {
                    (x as f64)
                        - if self.integer.exclusive_minimum {
                            1.0
                        } else {
                            0.0
                        }
                }),
                max: self.integer.maximum.map(|x| {
                    (x as f64)
                        + if self.integer.exclusive_maximum {
                            1.0
                        } else {
                            0.0
                        }
                }),
            });
        prompt.help_message = self.description;

        prompt
    }
}

impl<'a> Prompt for IntegerPrompt<'a> {
    fn prompt(&self) -> Result<Value> {
        let select = self.create_select_prompt();

        if let Some(select) = select {
            return select
                .with_render_config(render_config())
                .prompt()
                .with_context(|| format!("Failed to get {}", self.message));
        };

        let text = self.create_text_prompt();

        text.with_render_config(render_config())
            .prompt()
            .with_context(|| format!("Failed to get {}", self.message))
    }

    fn prompt_skippable(&self) -> Result<Option<Value>> {
        let select = self.create_select_prompt();

        if let Some(select) = select {
            return select
                .with_render_config(render_config_with_skkipable())
                .prompt_skippable()
                .with_context(|| format!("Failed to get {}", self.message));
        };

        let text = self.create_text_prompt();

        text.with_render_config(render_config_with_skkipable())
            .prompt_skippable()
            .with_context(|| format!("Failed to get {}", self.message))
    }
}
