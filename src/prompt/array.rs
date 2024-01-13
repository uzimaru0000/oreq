use color_eyre::owo_colors::{colors::css::Yellow, OwoColorize};
use inquire::{error::InquireResult, InquireError};
use openapiv3::{ArrayType, OpenAPI};
use serde_json::{json, Value};

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

    fn prompt_item(&self, is_required: bool) -> InquireResult<Value> {
        if let Some(items) = self.array.to_owned().items {
            let items = items.unbox();
            let items = items
                .item(self.api)
                .map_err(|x| InquireError::Custom(x.into()))?;
            let (items, _, description) = flat_schema(items, self.api, is_required)
                .map_err(|x| InquireError::Custom(x.into()))?;
            let mut values = Vec::new();

            eprintln!("{}", " Press esc to exit ".bg::<Yellow>());
            for idx in 0.. {
                let description = description.as_deref();
                let msg = format!("{}[{}]", self.message, idx);
                let prompt = SchemaPrompt::new(&msg, description, &items, self.api);
                let res = prompt.prompt_skippable()?;

                if let Some(res) = res {
                    values.push(res);
                } else {
                    break;
                }
            }

            Ok(json!(values))
        } else {
            Ok(json!([]))
        }
    }
}

impl<'a> Prompt for ArrayPrompt<'a> {
    fn prompt(&self) -> InquireResult<Value> {
        self.prompt_item(true)
    }

    fn prompt_skippable(&self) -> InquireResult<Option<Value>> {
        self.prompt_item(false).map(Some)
    }
}

#[cfg(test)]
#[cfg(feature = "manual")]
mod tests {
    use indoc::indoc;
    use openapiv3::{ArrayType, OpenAPI};

    use crate::prompt::Prompt;

    use super::ArrayPrompt;

    #[test]
    fn test_array_prompt_simple() {
        let schema = indoc! {"
            type: array
            items:
                type: string
        "};
        let arr = serde_yaml::from_str::<ArrayType>(schema).unwrap();
        let api = OpenAPI::default();
        let schema = ArrayPrompt::new("Test", &arr, &api);
        let val = schema.prompt().unwrap();
        assert!(val.is_array())
    }

    #[test]
    fn test_array_prompt_complex() {
        let schema = indoc! {"
            type: array
            items:
                type: object
                required:
                    - name
                    - age
                properties:
                    name:
                        type: string
                    age:
                        type: integer
        "};
        let arr = serde_yaml::from_str::<ArrayType>(schema).unwrap();
        let api = OpenAPI::default();
        let schema = ArrayPrompt::new("Test", &arr, &api);
        let val = schema.prompt().unwrap();
        assert!(val.is_array())
    }
}
