use anyhow::{Ok, Result};
use openapiv3::{ArrayType, OpenAPI};
use serde_json::Value;

use crate::schema::{flat_schema, ReferenceOrExt};

use super::{schema::SchemaPrompt, Prompt};

pub struct ArrayPrompt<'a> {
    message: &'a str,
    array: &'a ArrayType,
    api: &'a OpenAPI,
}

impl<'a> ArrayPrompt<'a> {
    pub fn new(message: &'a str, array: &'a ArrayType, api: &'a OpenAPI) -> Self {
        Self {
            message,
            array,
            api,
        }
    }

    fn prompt_item(&self, is_required: bool) -> Result<Vec<Value>> {
        if let Some(items) = self.array.to_owned().items {
            let items = items.unbox();
            let items = items.item(&self.api)?;
            let (items, _) = flat_schema(items, self.api, is_required)?;
            let mut end = false;
            let mut values = Vec::new();

            while !end {
                let prompt = SchemaPrompt::new(self.message, &items, self.api);
                let res = prompt.prompt_skippable()?;

                if let Some(res) = res {
                    values.push(res);
                } else {
                    end = true;
                }
            }

            Ok(values)
        } else {
            Ok(Vec::new())
        }
    }
}

impl<'a> Prompt<Vec<Value>> for ArrayPrompt<'a> {
    fn prompt(&self) -> Result<Vec<Value>> {
        self.prompt_item(true)
    }

    fn prompt_skippable(&self) -> Result<Option<Vec<Value>>> {
        self.prompt_item(false).map(|x| Some(x))
    }
}
