use openapiv3::{ArrayType, OpenAPI, ReferenceOr, Schema};
use promptuity::{
    event::{KeyCode, KeyModifiers},
    Prompt, PromptBody, PromptState, RenderPayload, Validator,
};
use serde_json::{json, Value};

use super::{
    prompt_builder,
    utils::{fmt_body, fmt_input},
};
use crate::schema::reference::ReferenceOrExt;

pub trait ArrayFormatter {
    fn fmt_prompt(&self, submitted: String, input: String) -> String;
    fn fmt_submitted_item(&self, idx: usize, value: Value) -> String;
    fn fmt_input(&self, payload: RenderPayload) -> String;
    fn fmt_submit(&self, value: Vec<Value>) -> String;
}

pub struct DefaultArrayFormatter;
impl ArrayFormatter for DefaultArrayFormatter {
    fn fmt_prompt(&self, submitted: String, input: String) -> String {
        format!("{}{}", submitted, input)
    }

    fn fmt_submitted_item(&self, _: usize, value: Value) -> String {
        format!("{},\n", value)
    }

    fn fmt_input(&self, payload: RenderPayload) -> String {
        let prompt_input = fmt_input(&payload.input);
        let prompt_body = fmt_body(&payload.body);

        format!("{}\n{}", prompt_input, prompt_body)
    }

    fn fmt_submit(&self, value: Vec<Value>) -> String {
        let value = value
            .iter()
            .enumerate()
            .map(|(_, v)| format!("{}", v))
            .collect::<Vec<_>>()
            .join(",\n");

        format!("{}\n", value)
    }
}

pub struct PromptOption {
    items: Option<ReferenceOr<Box<Schema>>>,
    min_items: Option<usize>,
    max_items: Option<usize>,
    unique_items: bool,
}

impl PromptOption {
    pub fn new(schema: ArrayType) -> Self {
        Self {
            items: schema.items,
            min_items: schema.min_items,
            max_items: schema.max_items,
            unique_items: schema.unique_items,
        }
    }
}

impl Validator<Vec<Value>> for PromptOption {
    fn validate(&self, value: &Vec<Value>) -> Result<(), String> {
        if let Some(min) = self.min_items {
            if value.len() < min {
                return Err(format!("Array must have at least {} items", min));
            }
        }

        Ok(())
    }
}

pub struct Array {
    message: String,
    option: PromptOption,
    formatter: Box<dyn ArrayFormatter>,
    api: OpenAPI,
    value: Vec<Value>,
    current_prompt: Option<Box<dyn Prompt<Output = Value>>>,
}

impl Array {
    pub fn new(message: String, api: &OpenAPI, schema: ArrayType) -> Self {
        Self {
            message,
            option: PromptOption::new(schema),
            formatter: Box::new(DefaultArrayFormatter),
            api: api.clone(),
            value: Vec::new(),
            current_prompt: None,
        }
    }

    fn create_prompt(&mut self) -> Result<(), promptuity::Error> {
        match &self.option.items {
            Some(refs) => {
                let refs = refs.clone().unbox();
                let item = refs
                    .item(&self.api)
                    .map_err(|x| promptuity::Error::Config(x.to_string()))?;

                let idx = self.value.len();
                let msg = format!("{}[{}]", self.message, idx);
                let mut prompt = prompt_builder(&self.api, item, msg);
                prompt.setup()?;
                self.current_prompt = Some(prompt);

                Ok(())
            }
            None => Err(promptuity::Error::Config(
                "No items schema found".to_owned(),
            )),
        }
    }

    fn check_unique(&self, value: &Value) -> bool {
        if !self.option.unique_items {
            return true;
        }

        !self.value.contains(value)
    }
}

impl Prompt for Array {
    type Output = Value;

    fn setup(&mut self) -> Result<(), promptuity::Error> {
        self.create_prompt()
    }

    fn handle(&mut self, code: KeyCode, modifiers: KeyModifiers) -> PromptState {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (code, modifiers) {
            return PromptState::Cancel;
        }

        let prompt = self.current_prompt.as_mut().unwrap();

        let state = prompt.handle(code, modifiers);
        match state {
            PromptState::Submit => {
                let is_valid = prompt.validate();
                if let Err(err) = is_valid {
                    return PromptState::Error(err);
                }

                let value = prompt.submit();
                if self.check_unique(&value) {
                    return PromptState::Error("Value already exists".to_owned());
                }
                if let Some(max) = self.option.max_items {
                    if self.value.len() >= max {
                        return PromptState::Error("Array is full".to_owned());
                    }
                }

                self.value.push(value);

                match self.create_prompt() {
                    Ok(_) => PromptState::Active,
                    Err(err) => PromptState::Fatal(err.to_string()),
                }
            }
            PromptState::Cancel => PromptState::Submit,
            _ => state,
        }
    }

    fn submit(&mut self) -> Self::Output {
        json!(self.value)
    }

    fn render(&mut self, state: &PromptState) -> Result<RenderPayload, String> {
        match state {
            PromptState::Submit => Ok(RenderPayload::new(self.message.clone(), None, None).body(
                PromptBody::Raw(self.formatter.fmt_submit(self.value.clone())),
            )),
            _ => {
                let prompt = self.current_prompt.as_mut().unwrap();

                let prompt_payload = prompt.render(state)?;
                let input = self.formatter.fmt_input(prompt_payload);

                let submitted = self
                    .value
                    .iter()
                    .enumerate()
                    .map(|(idx, value)| self.formatter.fmt_submitted_item(idx, value.clone()))
                    .collect::<Vec<_>>()
                    .join("");

                let payload = RenderPayload::new(self.message.clone(), None, None)
                    .body(PromptBody::Raw(self.formatter.fmt_prompt(submitted, input)));

                Ok(payload)
            }
        }
    }

    fn validate(&self) -> Result<(), String> {
        self.option.validate(&self.value)
    }
}
