use anyhow::{Context, Result};
use inquire::{CustomType, Select};
use openapiv3::NumberType;

use crate::prompt::validator::RangeValidator;

use super::{
    config::{render_config, render_config_with_skkipable},
    Prompt,
};

pub struct NumberPrompt<'a> {
    message: &'a str,
    number: &'a NumberType,
}

impl<'a> NumberPrompt<'a> {
    pub fn new(message: &'a str, number: &'a NumberType) -> Self {
        Self { message, number }
    }

    fn create_select_prompt(&self) -> Option<Select<f64>> {
        let enumeration = self
            .number
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

    fn create_text_prompt(&self) -> CustomType<f64> {
        CustomType::new(self.message).with_validator(RangeValidator {
            min: self.number.minimum.map(|x| {
                x - if self.number.exclusive_minimum {
                    1.0
                } else {
                    0.0
                }
            }),
            max: self.number.maximum.map(|x| {
                x + if self.number.exclusive_maximum {
                    1.0
                } else {
                    0.0
                }
            }),
        })
    }
}

impl<'a> Prompt<f64> for NumberPrompt<'a> {
    fn prompt(&self) -> Result<f64> {
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

    fn prompt_skippable(&self) -> Result<Option<f64>> {
        let select = self.create_select_prompt();

        if let Some(select) = select {
            return select
                .with_render_config(render_config_with_skkipable())
                .prompt_skippable()
                .with_context(|| format!("Failed to get {}", self.message));
        };

        self.create_text_prompt()
            .with_render_config(render_config_with_skkipable())
            .prompt_skippable()
            .with_context(|| format!("Failed to get {}", self.message))
    }
}
