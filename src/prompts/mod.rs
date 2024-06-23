use array::Array;
use boolean::Boolean;
use indexmap::IndexMap;
use number::Number;
use object::Object;
use openapiv3::{OpenAPI, Schema, SchemaKind, Type};
use promptuity::Prompt;
use serde_json::Value;
use skippable::Skippable;
use string::StringPrompt;

pub mod array;
pub mod boolean;
pub mod enumeration;
pub(crate) mod error;
pub mod number;
pub mod object;
pub mod skippable;
pub mod string;
pub(crate) mod utils;

pub fn optional_prompt_builder(
    api: &OpenAPI,
    schema: &Schema,
    message: String,
    hint: Option<String>,
    default: Option<IndexMap<String, Value>>,
) -> Box<dyn Prompt<Output = Option<Value>>> {
    match &schema.schema_kind {
        SchemaKind::Type(Type::Boolean(_)) => {
            let mut p = Boolean::new(message);
            if let Some(hint) = hint {
                p.with_hint(hint);
            };

            Box::new(Skippable::new(p))
        }
        SchemaKind::Type(Type::String(string)) => {
            let mut p = StringPrompt::new(message, string.clone().into());
            if let Some(hint) = hint {
                p.with_hint(hint);
            };

            Box::new(Skippable::new(p))
        }
        SchemaKind::Type(Type::Number(number)) => {
            let mut p = Number::new(message, number.clone().into());
            if let Some(hint) = hint {
                p.with_hint(hint);
            };

            Box::new(Skippable::new(p))
        }
        SchemaKind::Type(Type::Integer(integer)) => {
            let mut p = Number::new(message, integer.clone().into());
            if let Some(hint) = hint {
                p.with_hint(hint);
            };

            Box::new(Skippable::new(p))
        }
        SchemaKind::Type(Type::Object(object)) => {
            let mut object = Object::new(message, api, object.clone());
            if let Some(default) = default {
                object.with_value(default);
            }

            Box::new(Skippable::new(object))
        }
        SchemaKind::Type(Type::Array(array)) => {
            Box::new(Skippable::new(Array::new(message, api, array.clone())))
        }
        _ => unimplemented!(),
    }
}

pub fn prompt_builder(
    api: &OpenAPI,
    schema: &Schema,
    message: String,
    hint: Option<String>,
    default: Option<IndexMap<String, Value>>,
) -> Box<dyn Prompt<Output = Value>> {
    match &schema.schema_kind {
        SchemaKind::Type(Type::Boolean(_)) => {
            let mut p = Boolean::new(message);
            if let Some(hint) = hint {
                p.with_hint(hint);
            };
            Box::new(p)
        }
        SchemaKind::Type(Type::String(string)) => {
            let mut p = StringPrompt::new(message, string.clone().into());
            if let Some(hint) = hint {
                p.with_hint(hint);
            };
            Box::new(p)
        }
        SchemaKind::Type(Type::Number(number)) => {
            let mut p = Number::new(message, number.clone().into());
            if let Some(hint) = hint {
                p.with_hint(hint);
            };
            Box::new(p)
        }
        SchemaKind::Type(Type::Integer(integer)) => {
            let mut p = Number::new(message, integer.clone().into());
            if let Some(hint) = hint {
                p.with_hint(hint);
            };
            Box::new(p)
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
