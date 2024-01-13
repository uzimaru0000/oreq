use std::str::FromStr;

use inquire::{error::InquireResult, CustomType, Select};
use openapiv3::{IntegerType, NumberType};
use serde_json::{json, Number, Value};

use crate::prompt::validator::RangeValidator;

use super::{
    config::{render_config, render_config_with_skkipable},
    Prompt,
};

#[derive(Debug, Clone)]
pub struct NumType<T>
where
    T: FromStr + serde::Serialize,
{
    pub multiple_of: Option<Number>,
    pub exclusive_minimum: bool,
    pub exclusive_maximum: bool,
    pub minimum: Option<Number>,
    pub maximum: Option<Number>,
    pub enumeration: Vec<Option<Number>>,
    _phantom: std::marker::PhantomData<T>,
}
impl<T: FromStr + serde::Serialize> From<NumberType> for NumType<T> {
    fn from(number: NumberType) -> Self {
        Self {
            multiple_of: number.multiple_of.and_then(Number::from_f64),
            exclusive_minimum: number.exclusive_minimum,
            exclusive_maximum: number.exclusive_maximum,
            minimum: number.minimum.and_then(Number::from_f64),
            maximum: number.maximum.and_then(Number::from_f64),
            enumeration: number
                .enumeration
                .iter()
                .map(|x| x.and_then(Number::from_f64))
                .collect(),
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<T: FromStr + serde::Serialize> From<IntegerType> for NumType<T> {
    fn from(integer: IntegerType) -> Self {
        Self {
            multiple_of: integer.multiple_of.and_then(|x| Number::from_f64(x as f64)),
            exclusive_minimum: integer.exclusive_minimum,
            exclusive_maximum: integer.exclusive_maximum,
            minimum: integer.minimum.and_then(|x| Number::from_f64(x as f64)),
            maximum: integer.maximum.and_then(|x| Number::from_f64(x as f64)),
            enumeration: integer
                .enumeration
                .iter()
                .map(|x| x.and_then(|x| Number::from_f64(x as f64)))
                .collect(),
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct NumberPrompt<'a, T: FromStr + serde::Serialize> {
    message: &'a str,
    description: Option<&'a str>,
    number: NumType<T>,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T: FromStr + serde::Serialize> NumberPrompt<'a, T> {
    pub fn new(message: &'a str, description: Option<&'a str>, number: NumType<T>) -> Self {
        Self {
            message,
            description,
            number,
            _phantom: std::marker::PhantomData,
        }
    }

    fn create_select_prompt(&self) -> Option<Select<Value>> {
        let enumeration = self
            .number
            .enumeration
            .iter()
            .map(|x| x.clone().and_then(|x| serde_json::to_value(x).ok()))
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
            .with_parser(&|x| x.parse::<T>().map(|x| json!(x)).map_err(|_| ()))
            .with_validator(RangeValidator {
                min: self.number.minimum.clone().and_then(|x| {
                    let min = x.as_f64()?;
                    Some(
                        min - if self.number.exclusive_minimum {
                            1.0
                        } else {
                            0.0
                        },
                    )
                }),
                max: self.number.maximum.clone().and_then(|x| {
                    let max = x.as_f64()?;
                    Some(
                        max + if self.number.exclusive_maximum {
                            1.0
                        } else {
                            0.0
                        },
                    )
                }),
            });
        prompt.help_message = self.description;

        prompt
    }
}

impl<'a, T: FromStr + serde::Serialize> Prompt for NumberPrompt<'a, T> {
    fn prompt(&self) -> InquireResult<Value> {
        let select = self.create_select_prompt();

        if let Some(select) = select {
            select.with_render_config(render_config()).prompt()
        } else {
            self.create_prompt()
                .with_render_config(render_config())
                .prompt()
        }
    }

    fn prompt_skippable(&self) -> InquireResult<Option<Value>> {
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
    }
}

#[cfg(test)]
#[cfg(feature = "manual")]
mod tests {
    use indoc::indoc;

    use super::*;
    use crate::prompt::Prompt;

    #[test]
    fn test_number_prompt_type_number() {
        let schema = indoc! {"
            type: number
        "};
        let schema = serde_yaml::from_str::<NumberType>(schema).unwrap();

        let prompt = NumberPrompt::<f64>::new("test", Some("input 2.0"), schema.into());

        let v = prompt.prompt().unwrap();
        assert_eq!(v, json!(2.0));
    }

    #[test]
    fn test_number_prompt_type_integer() {
        let schema = indoc! {"
            type: integer
        "};
        let schema = serde_yaml::from_str::<IntegerType>(schema).unwrap();

        let prompt = NumberPrompt::<i64>::new("test", Some("input 2"), schema.into());

        let v = prompt.prompt().unwrap();
        assert_eq!(v, json!(2));
    }
}
