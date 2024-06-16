use array::Array;
use boolean::Boolean;
use indexmap::IndexMap;
use number::Number;
use object::Object;
use openapiv3::{OpenAPI, Schema, SchemaKind, Type};
use promptuity::Prompt;
use serde_json::Value;
use string::StringPrompt;

pub mod array;
pub mod boolean;
pub mod enumeration;
pub(crate) mod error;
pub mod number;
pub mod object;
pub mod string;
pub(crate) mod utils;

pub fn prompt_builder(
    api: &OpenAPI,
    schema: &Schema,
    message: String,
    default: Option<IndexMap<String, Value>>,
) -> Box<dyn Prompt<Output = Value>> {
    match &schema.schema_kind {
        SchemaKind::Type(Type::Boolean(_)) => Box::new(Boolean::new(message)),
        SchemaKind::Type(Type::String(string)) => {
            Box::new(StringPrompt::new(message, string.clone().into()))
        }
        SchemaKind::Type(Type::Number(number)) => {
            Box::new(Number::new(message, number.clone().into()))
        }
        SchemaKind::Type(Type::Integer(integer)) => {
            Box::new(Number::new(message, integer.clone().into()))
        }
        SchemaKind::Type(Type::Object(object)) => {
            let mut object = Object::new(message, api, object.clone());
            if let Some(default) = default {
                object.with_value(default);
            }

            Box::new(object)
        }
        SchemaKind::Type(Type::Array(array)) => Box::new(Array::new(message, api, array.clone())),
        _ => unimplemented!(),
    }
}
