use std::collections::VecDeque;

use indexmap::IndexMap;
use openapiv3::{AdditionalProperties, ObjectType, OpenAPI, ReferenceOr, Schema};
use promptuity::{
    event::{KeyCode, KeyModifiers},
    Prompt, PromptBody, PromptState, RenderPayload,
};
use serde_json::{json, Value};

use crate::schema::reference::ReferenceOrExt;

use super::utils::{fmt_body, fmt_input};

use super::prompt_builder;

pub trait ObjectFormatter {
    fn fmt_prompt(&self, submitted: String, input: String) -> String;
    fn fmt_submitted_item(&self, key: String, value: Option<Value>) -> String;
    fn fmt_input(&self, key: String, payload: RenderPayload, is_required: bool) -> String;
    fn fmt_submit(&self, value: IndexMap<String, Value>) -> String;
}

pub struct DefaultObjectFormatter;
impl ObjectFormatter for DefaultObjectFormatter {
    fn fmt_prompt(&self, submitted: String, input: String) -> String {
        format!("{}{}", submitted, input)
    }

    fn fmt_submitted_item(&self, key: String, value: Option<Value>) -> String {
        match value {
            Some(value) => format!("{}: {}\n", key, value),
            None => format!("{}: undefined\n", key),
        }
    }

    fn fmt_input(&self, key: String, payload: RenderPayload, is_required: bool) -> String {
        let prompt_input = fmt_input(&payload.input);
        let prompt_body = fmt_body(&payload.body);
        let required = if !is_required { "?" } else { "" };

        format!("{key}{required}: {prompt_input}\n{prompt_body}")
    }

    fn fmt_submit(&self, value: IndexMap<String, Value>) -> String {
        let value = value
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        format!("{}\n", value)
    }
}

struct PromptOption {
    properties: IndexMap<String, ReferenceOr<Box<Schema>>>,
    required: Vec<String>,
    _additional_properties: Option<AdditionalProperties>,
    _min_properties: Option<usize>,
    _max_properties: Option<usize>,
}

impl PromptOption {
    pub fn new(schema: ObjectType) -> Self {
        Self {
            properties: schema.properties,
            required: schema.required,
            _additional_properties: schema.additional_properties,
            _min_properties: schema.min_properties,
            _max_properties: schema.max_properties,
        }
    }

    fn has_required(&self, key: String) -> bool {
        self.required.contains(&key)
    }
}

pub struct Object {
    message: String,
    option: PromptOption,
    api: OpenAPI,
    formatter: Box<dyn ObjectFormatter>,
    value: IndexMap<String, Option<Value>>,
    prompts: VecDeque<(String, Box<dyn Prompt<Output = Value>>)>,
    current_prompt: Option<(String, Box<dyn Prompt<Output = Value>>)>,
}

impl Object {
    pub fn new(message: String, api: &OpenAPI, schema: ObjectType) -> Self {
        Self {
            message,
            option: PromptOption::new(schema),
            api: api.clone(),
            formatter: Box::new(DefaultObjectFormatter),
            value: IndexMap::new(),
            prompts: VecDeque::new(),
            current_prompt: None,
        }
    }

    pub fn with_formatter(&mut self, formatter: impl ObjectFormatter + 'static) -> &mut Self {
        self.formatter = Box::new(formatter);
        self
    }

    pub fn with_value(&mut self, value: IndexMap<String, Value>) -> &mut Self {
        self.value = value
            .into_iter()
            .map(|(k, v)| (k, Some(v)))
            .collect::<IndexMap<_, _>>();
        self
    }

    fn next_prompt(&mut self) -> Result<bool, promptuity::Error> {
        let mut prompt = self.prompts.pop_front();
        if let Some((_, prompt)) = &mut prompt {
            prompt.setup()?;
        }

        self.current_prompt = prompt;
        Ok(self.current_prompt.is_some())
    }

    fn get_value(&self) -> IndexMap<String, Value> {
        self.value
            .clone()
            .into_iter()
            .filter_map(|(k, v)| v.map(|x| (k.clone(), x.clone())))
            .collect::<IndexMap<_, _>>()
    }
}

impl Prompt for Object {
    type Output = Value;

    fn setup(&mut self) -> Result<(), promptuity::Error> {
        let properties = self.option.properties.clone();
        let properties = properties
            .into_iter()
            .filter(|(k, _)| !self.value.contains_key(k));

        for (key, schema) in properties {
            let schema = schema.unbox();
            let schema = schema
                .item(&self.api)
                .map_err(|x| promptuity::Error::Config(x.to_string()))?;

            let prompt = prompt_builder(&self.api, schema, key.clone(), None);
            self.prompts.push_back((key.clone(), prompt));
        }

        if !self.next_prompt()? {
            return Err(promptuity::Error::Config("No prompt found".to_string()));
        }

        Ok(())
    }

    fn handle(&mut self, code: KeyCode, modifiers: KeyModifiers) -> PromptState {
        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (code, modifiers) {
            return PromptState::Cancel;
        }

        let (key, prompt) = self.current_prompt.as_mut().unwrap();

        let state = prompt.handle(code, modifiers);
        match state {
            PromptState::Submit => {
                let is_valid = prompt.validate();
                if let Err(err) = is_valid {
                    return PromptState::Error(err);
                }

                let value = prompt.submit();
                self.value.insert(key.clone(), Some(value));

                let init = self.next_prompt();
                match init {
                    Ok(false) => PromptState::Submit,
                    Ok(true) => PromptState::Active,
                    Err(err) => PromptState::Fatal(err.to_string()),
                }
            }
            PromptState::Cancel => {
                if self.option.has_required(key.clone()) {
                    PromptState::Error(format!("{} is required field", key))
                } else {
                    self.value.insert(key.clone(), None);

                    let init = self.next_prompt();
                    match init {
                        Ok(false) => PromptState::Submit,
                        Ok(true) => PromptState::Active,
                        Err(err) => PromptState::Fatal(err.to_string()),
                    }
                }
            }
            _ => state,
        }
    }

    fn submit(&mut self) -> Self::Output {
        json!(self.get_value())
    }

    fn render(
        &mut self,
        state: &promptuity::PromptState,
    ) -> Result<promptuity::RenderPayload, String> {
        match state {
            PromptState::Submit => Ok(RenderPayload::new(self.message.clone(), None, None)
                .body(PromptBody::Raw(self.formatter.fmt_submit(self.get_value())))),
            _ => {
                let (key, prompt) = self.current_prompt.as_mut().unwrap();

                let prompt_payload = prompt.render(state)?;
                let input = self.formatter.fmt_input(
                    key.clone(),
                    prompt_payload,
                    self.option.has_required(key.clone()),
                );

                let submitted = self
                    .value
                    .iter()
                    .map(|(key, value)| {
                        self.formatter
                            .fmt_submitted_item(key.clone(), value.clone())
                    })
                    .collect::<Vec<_>>()
                    .join("");

                let payload = RenderPayload::new(self.message.clone(), None, None)
                    .body(PromptBody::Raw(self.formatter.fmt_prompt(submitted, input)));

                Ok(payload)
            }
        }
    }
}
