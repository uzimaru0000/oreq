use anyhow::Result;
use inquire::Confirm;
use openapiv3::OpenAPI;
use serde_json::Value;

use crate::schema::SchemaType;

use super::{
    array::ArrayPrompt, boolean::BooleanPrompt, integer::IntegerPrompt, number::NumberPrompt,
    string::StringPrompt, Prompt,
};

pub struct SchemaPrompt<'a> {
    message: &'a str,
    schema: &'a SchemaType,
    api: &'a OpenAPI,
}

impl<'a> SchemaPrompt<'a> {
    pub fn new(message: &'a str, schema: &'a SchemaType, api: &'a OpenAPI) -> Self {
        Self {
            message,
            schema,
            api,
        }
    }
}

impl<'a> Prompt<Value> for SchemaPrompt<'a> {
    fn prompt(&self) -> Result<Value> {
        match self.schema {
            SchemaType::String(t) => StringPrompt::new(self.message, t, true)
                .prompt()
                .map(|x| x.into()),
            SchemaType::Number(t) => NumberPrompt::new(self.message, t)
                .prompt()
                .map(|x| x.into()),
            SchemaType::Integer(t) => IntegerPrompt::new(self.message, t)
                .prompt()
                .map(|x| x.into()),
            SchemaType::Boolean(_) => BooleanPrompt::new(self.message).prompt().map(|x| x.into()),
            SchemaType::Object(t) => {
                let mut obj = serde_json::Map::new();

                for (k, (v, is_req)) in t {
                    let prompt = SchemaPrompt::new(k, v, self.api);
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
            SchemaType::Array(t) => ArrayPrompt::new(self.message, t, self.api)
                .prompt()
                .map(|x| x.into()),
        }
    }

    fn prompt_skippable(&self) -> Result<Option<Value>> {
        match self.schema {
            SchemaType::String(t) => StringPrompt::new(self.message, t, true)
                .prompt_skippable()
                .map(|x| x.map(|x| x.into())),
            SchemaType::Number(t) => NumberPrompt::new(self.message, t)
                .prompt_skippable()
                .map(|x| x.map(|x| x.into())),
            SchemaType::Integer(t) => IntegerPrompt::new(self.message, t)
                .prompt_skippable()
                .map(|x| x.map(|x| x.into())),
            SchemaType::Boolean(_) => BooleanPrompt::new(self.message)
                .prompt_skippable()
                .map(|x| x.map(|x| x.into())),
            SchemaType::Object(t) => {
                let is_continue =
                    Confirm::new(format!("Continue to input {}?", self.message).as_str())
                        .prompt()?;
                if !is_continue {
                    return Ok(None);
                }

                let mut obj = serde_json::Map::new();
                for (k, (v, is_req)) in t {
                    let prompt = SchemaPrompt::new(k, v, self.api);
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

                Ok(Some(obj.into()))
            }
            SchemaType::Array(t) => ArrayPrompt::new(self.message, t, self.api)
                .prompt_skippable()
                .map(|x| x.map(|x| x.into())),
        }
    }
}
