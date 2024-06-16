use std::fmt::Display;

use serde_json::Value;

pub struct SerdeValue(Value);

impl From<Value> for SerdeValue {
    fn from(value: Value) -> Self {
        Self(value)
    }
}

impl Display for SerdeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match &self.0 {
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.to_owned(),
            Value::Array(a) => a
                .iter()
                .map::<SerdeValue, _>(|x| x.clone().into())
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(","),
            Value::Null => String::new(),
            _ => String::new(),
        };

        write!(f, "{}", string)
    }
}
