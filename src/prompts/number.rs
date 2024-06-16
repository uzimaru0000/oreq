use std::str::FromStr;

use num_traits::Num;
use openapiv3::{IntegerType, NumberType};
use promptuity::{
    event::{KeyCode, KeyModifiers},
    prompts::{NumberFormatter, SelectOption},
    Prompt, PromptState, RenderPayload, Validator,
};
use serde::Serialize;
use serde_json::{json, Value};

use super::enumeration::Enumeration;

#[derive(Clone)]
pub struct PromptOption<N> {
    multiple_of: Option<N>,
    exclusive_minimum: bool,
    exclusive_maximum: bool,
    minimum: Option<N>,
    maximum: Option<N>,
    enumeration: Vec<Option<N>>,
}

impl From<NumberType> for PromptOption<f64> {
    fn from(value: NumberType) -> Self {
        Self {
            multiple_of: value.multiple_of,
            exclusive_minimum: value.exclusive_minimum,
            exclusive_maximum: value.exclusive_maximum,
            minimum: value.minimum,
            maximum: value.maximum,
            enumeration: value.enumeration,
        }
    }
}

impl From<IntegerType> for PromptOption<i64> {
    fn from(value: IntegerType) -> Self {
        Self {
            multiple_of: value.multiple_of,
            exclusive_minimum: value.exclusive_minimum,
            exclusive_maximum: value.exclusive_maximum,
            minimum: value.minimum,
            maximum: value.maximum,
            enumeration: value.enumeration,
        }
    }
}

fn is_multiple_of<N>(value: N, multiple_of: N) -> bool
where
    N: Num,
{
    value.rem(multiple_of) == N::zero()
}

fn is_minimum<N>(value: N, minimum: N, exclusive: bool) -> bool
where
    N: PartialOrd,
{
    if exclusive {
        value > minimum
    } else {
        value >= minimum
    }
}

fn is_maximum<N>(value: N, maximum: N, exclusive: bool) -> bool
where
    N: PartialOrd,
{
    if exclusive {
        value < maximum
    } else {
        value <= maximum
    }
}

impl<N> Validator<String> for PromptOption<N>
where
    N: Num + PartialOrd + Clone + FromStr + ToString,
{
    fn validate(&self, value: &String) -> Result<(), String> {
        let value = value
            .parse::<N>()
            .map_err(|_| "Value is not a number".to_string())?;

        let range = match (self.minimum.clone(), self.maximum.clone()) {
            (Some(min), Some(max)) => {
                if is_maximum(&value, &max, self.exclusive_maximum)
                    && is_minimum(&value, &min, self.exclusive_minimum)
                {
                    Ok(())
                } else {
                    Err(format!(
                        "Value must be between {} and {}",
                        min.to_string(),
                        max.to_string()
                    ))
                }
            }
            (Some(min), None) => {
                if is_minimum(&value, &min, self.exclusive_minimum) {
                    Ok(())
                } else {
                    Err(format!(
                        "Value must be greater than or equal to {}",
                        min.to_string()
                    ))
                }
            }
            (None, Some(max)) => {
                if is_maximum(&value, &max, self.exclusive_maximum) {
                    Ok(())
                } else {
                    Err(format!(
                        "Value must be less than or equal to {}",
                        max.to_string()
                    ))
                }
            }
            _ => Ok(()),
        };

        let multiple = self.multiple_of.clone().map_or(Ok(()), |multiple| {
            if is_multiple_of(value.clone(), multiple.clone()) {
                Ok(())
            } else {
                Err(format!(
                    "Value must be a multiple of {}",
                    multiple.to_string()
                ))
            }
        });

        range.and(multiple)
    }
}

pub struct Number {
    original: promptuity::prompts::Number,
    enumeration: Option<Enumeration<Value>>,
}

impl Number {
    pub fn new<T>(message: String, option: PromptOption<T>) -> Self
    where
        T: Num
            + PartialOrd
            + Clone
            + Default
            + FromStr
            + ToString
            + Serialize
            + 'static,
    {
        let enumeration = if option.enumeration.is_empty() {
            None
        } else {
            let options = option
                .enumeration
                .clone()
                .into_iter()
                .flatten()
                .map(|value| {
                    let label = value.to_string();
                    SelectOption::new(label, json!(value))
                })
                .collect();

            Some(Enumeration::new(message.clone(), options))
        };
        let mut original = promptuity::prompts::Number::new(message.clone());
        original.with_validator(option.clone());

        Self {
            original,
            enumeration,
        }
    }

    pub fn with_formatter(&mut self, formatter: impl NumberFormatter + 'static) -> &mut Self {
        self.original.with_formatter(formatter);
        self
    }

    pub fn with_hint(&mut self, hint: impl std::fmt::Display) -> &mut Self {
        self.original.with_hint(hint);
        self
    }

    pub fn with_placeholder(&mut self, placeholder: impl std::fmt::Display) -> &mut Self {
        self.original.with_placeholder(placeholder);
        self
    }
}

impl Prompt for Number {
    type Output = Value;

    fn setup(&mut self) -> Result<(), promptuity::Error> {
        if let Some(enumeration) = &mut self.enumeration {
            return enumeration.setup();
        }

        self.original.setup()
    }

    fn handle(&mut self, code: KeyCode, modifiers: KeyModifiers) -> PromptState {
        if let Some(enumeration) = &mut self.enumeration {
            return enumeration.handle(code, modifiers);
        }

        self.original.handle(code, modifiers)
    }

    fn submit(&mut self) -> Self::Output {
        if let Some(enumeration) = &mut self.enumeration {
            return enumeration.submit();
        }

        json!(self.original.submit())
    }

    fn render(&mut self, state: &PromptState) -> Result<RenderPayload, String> {
        if let Some(enumeration) = &mut self.enumeration {
            return enumeration.render(state);
        }

        self.original.render(state)
    }

    fn validate(&self) -> Result<(), String> {
        if let Some(enumeration) = &self.enumeration {
            return enumeration.validate();
        }

        self.original.validate()
    }
}
