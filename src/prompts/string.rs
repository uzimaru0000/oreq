use openapiv3::{StringFormat, StringType, VariantOrUnknownOrEmpty};
use promptuity::{
    event::{KeyCode, KeyModifiers},
    prompts::{Input, Password, SelectOption},
    Prompt, PromptState, RenderPayload, Validator,
};
use serde_json::Value;

use super::enumeration::Enumeration;

#[derive(Clone)]
pub struct PromptOption {
    format: VariantOrUnknownOrEmpty<StringFormat>,
    pattern: Option<String>,
    enumeration: Vec<Option<String>>,
    min_length: Option<usize>,
    max_length: Option<usize>,
}

impl From<StringType> for PromptOption {
    fn from(string_type: StringType) -> Self {
        Self {
            format: string_type.format,
            pattern: string_type.pattern,
            enumeration: string_type.enumeration,
            min_length: string_type.min_length,
            max_length: string_type.max_length,
        }
    }
}

impl Validator<String> for PromptOption {
    fn validate(&self, value: &String) -> Result<(), String> {
        if let Some(pattern) = &self.pattern {
            let re = regex::Regex::new(pattern).unwrap();
            if !re.is_match(value) {
                return Err(format!("Value does not match pattern: {}", pattern));
            }
        }

        if let Some(min_length) = self.min_length {
            if value.len() < min_length {
                return Err(format!(
                    "Value is too short. Minimum length is {}",
                    min_length
                ));
            }
        }

        if let Some(max_length) = self.max_length {
            if value.len() > max_length {
                return Err(format!(
                    "Value is too long. Maximum length is {}",
                    max_length
                ));
            }
        }

        Ok(())
    }
}

pub struct StringPrompt {
    original: Input,
    enumeration: Option<Enumeration<Value>>,
    password: Option<Password>,
}

impl StringPrompt {
    pub fn new(message: String, option: PromptOption) -> Self {
        let enumeration = if !option.enumeration.is_empty() {
            let options = option
                .enumeration
                .clone()
                .into_iter()
                .flatten()
                .map(|value| {
                    let label = value.to_string();
                    SelectOption::new(label, Value::String(value))
                })
                .collect();

            Some(Enumeration::new(message.clone(), options))
        } else {
            None
        };

        let password = if let VariantOrUnknownOrEmpty::Item(StringFormat::Password) = option.format
        {
            let mut password = Password::new(message.clone());
            password.with_validator(option.clone());

            Some(password)
        } else {
            None
        };

        let mut original = Input::new(message.clone());
        original.with_validator(option.clone());

        Self {
            original,
            enumeration,
            password,
        }
    }

    pub fn with_hint(&mut self, hint: impl std::fmt::Display) -> &mut Self {
        self.original.with_hint(hint);
        self
    }
}

impl Prompt for StringPrompt {
    type Output = Value;

    fn setup(&mut self) -> Result<(), promptuity::Error> {
        if let Some(enumeration) = &mut self.enumeration {
            return enumeration.setup();
        } else if let Some(password) = &mut self.password {
            return password.setup();
        }

        self.original.setup()
    }

    fn handle(&mut self, code: KeyCode, modifiers: KeyModifiers) -> PromptState {
        if let Some(enumeration) = &mut self.enumeration {
            return enumeration.handle(code, modifiers);
        } else if let Some(password) = &mut self.password {
            return password.handle(code, modifiers);
        }

        self.original.handle(code, modifiers)
    }

    fn submit(&mut self) -> Self::Output {
        if let Some(enumeration) = &mut self.enumeration {
            return enumeration.submit();
        } else if let Some(password) = &mut self.password {
            return Value::String(password.submit());
        }

        let value = self.original.submit();
        Value::String(value)
    }

    fn render(&mut self, state: &PromptState) -> Result<RenderPayload, String> {
        if let Some(enumeration) = &mut self.enumeration {
            return enumeration.render(state);
        } else if let Some(password) = &mut self.password {
            return password.render(state);
        }

        self.original.render(state)
    }

    fn validate(&self) -> Result<(), String> {
        if let Some(enumeration) = &self.enumeration {
            return enumeration.validate();
        } else if let Some(password) = &self.password {
            return password.validate();
        }

        self.original.validate()
    }
}
