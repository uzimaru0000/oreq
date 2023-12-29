use anyhow::{Context, Result};
use inquire::{CustomType, Select};
use openapiv3::IntegerType;

use crate::prompt::validator::RangeValidator;

use super::{
    config::{render_config, render_config_with_skkipable},
    Prompt,
};

pub struct IntegerPrompt<'a> {
    message: &'a str,
    integer: &'a IntegerType,
}

impl<'a> IntegerPrompt<'a> {
    pub fn new(message: &'a str, integer: &'a IntegerType) -> Self {
        Self { message, integer }
    }

    fn create_select_prompt(&self) -> Option<Select<i64>> {
        let enumeration = self
            .integer
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

    fn create_text_prompt(&self) -> CustomType<i64> {
        CustomType::new(self.message).with_validator(RangeValidator {
            min: self
                .integer
                .minimum
                .map(|x| x - if self.integer.exclusive_minimum { 1 } else { 0 }),
            max: self
                .integer
                .maximum
                .map(|x| x + if self.integer.exclusive_maximum { 1 } else { 0 }),
        })
    }
}

impl<'a> Prompt<i64> for IntegerPrompt<'a> {
    fn prompt(&self) -> Result<i64> {
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

    fn prompt_skippable(&self) -> Result<Option<i64>> {
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
