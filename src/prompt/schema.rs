use indexmap::IndexMap;
use inquire::{error::InquireResult, Confirm};
use openapiv3::OpenAPI;
use serde_json::Value;

use crate::schema::SchemaType;

use super::{
    array::ArrayPrompt, boolean::BooleanPrompt, number::NumberPrompt, string::StringPrompt, Prompt,
};

struct ObjectPrompt<'a> {
    message: &'a str,
    obj: &'a IndexMap<String, (SchemaType, bool, Option<String>)>,
    api: &'a OpenAPI,
    args_input: &'a Option<Value>,
}
impl<'a> ObjectPrompt<'a> {
    pub fn new(
        message: &'a str,
        obj: &'a IndexMap<String, (SchemaType, bool, Option<String>)>,
        api: &'a OpenAPI,
        args_input: &'a Option<Value>,
    ) -> Self {
        Self {
            message,
            obj,
            api,
            args_input,
        }
    }
}
impl<'a> Prompt for ObjectPrompt<'a> {
    fn prompt(&self) -> InquireResult<Value> {
        let mut obj = serde_json::Map::new();

        for (k, (v, is_req, description)) in self.obj {
            let input = self.args_input.as_ref().and_then(|x| x.get(k));
            if let Some(input) = input {
                let matched = match v {
                    SchemaType::String(_) => input.is_string(),
                    SchemaType::Number(_) => input.is_number(),
                    SchemaType::Integer(_) => input.is_i64(),
                    SchemaType::Boolean(_) => input.is_boolean(),
                    _ => false,
                };
                if matched {
                    obj.insert(k.to_owned(), input.clone());
                    continue;
                }
            }

            let description = description.as_ref().map(|x| x.as_str());
            let args_input = self.args_input.clone().and_then(|x| x.get(k).cloned());
            let prompt = SchemaPrompt::new(k, description, v, self.api, &args_input);
            if is_req.to_owned() {
                let v = prompt.prompt()?;
                obj.insert(k.to_owned(), v);
            } else {
                let v = prompt.prompt_skippable()?;
                if let Some(v) = v {
                    obj.insert(k.to_owned(), v);
                }
            };
        }

        Ok(obj.into())
    }

    fn prompt_skippable(&self) -> InquireResult<Option<Value>> {
        let is_continue =
            Confirm::new(format!("Continue to input {}?", self.message).as_str()).prompt()?;
        if !is_continue {
            return Ok(None);
        }

        let obj = self.prompt()?;

        Ok(Some(obj))
    }
}

pub struct SchemaPrompt<'a> {
    message: &'a str,
    description: Option<&'a str>,
    schema: &'a SchemaType,
    api: &'a OpenAPI,
    args_input: &'a Option<Value>,
}

impl<'a> SchemaPrompt<'a> {
    pub fn new(
        message: &'a str,
        description: Option<&'a str>,
        schema: &'a SchemaType,
        api: &'a OpenAPI,
        args_input: &'a Option<Value>,
    ) -> Self {
        Self {
            message,
            description,
            schema,
            api,
            args_input,
        }
    }

    fn create_prompt(&self) -> Box<dyn Prompt + 'a> {
        match self.schema {
            SchemaType::String(t) => Box::new(StringPrompt::new(self.message, self.description, t)),
            SchemaType::Number(t) => Box::new(NumberPrompt::<f64>::new(
                self.message,
                self.description,
                t.clone().into(),
            )),
            SchemaType::Integer(t) => Box::new(NumberPrompt::<i64>::new(
                self.message,
                self.description,
                t.clone().into(),
            )),
            SchemaType::Boolean(_) => Box::new(BooleanPrompt::new(self.message, self.description)),
            SchemaType::Object(t) => Box::new(ObjectPrompt::new(
                self.message,
                t,
                self.api,
                self.args_input,
            )),
            SchemaType::Array(t) => Box::new(ArrayPrompt::new(self.message, t, self.api)),
        }
    }
}

impl<'a> Prompt for SchemaPrompt<'a> {
    fn prompt(&self) -> InquireResult<Value> {
        self.create_prompt().prompt()
    }

    fn prompt_skippable(&self) -> InquireResult<Option<Value>> {
        self.create_prompt().prompt_skippable()
    }
}
