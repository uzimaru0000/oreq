use serde_json::Value;

pub struct SerdeValue(Value);

impl From<Value> for SerdeValue {
    fn from(value: Value) -> Self {
        Self(value)
    }
}

impl SerdeValue {
    pub fn to_query_string(&self) -> Option<String> {
        match &self.0 {
            Value::Bool(b) => Some(b.to_string()),
            Value::Number(n) => Some(n.to_string()),
            Value::String(s) => Some(s.to_owned()),
            Value::Array(a) => Some(
                a.iter()
                    .map::<SerdeValue, _>(|x| x.clone().into())
                    .filter_map(|x| x.to_query_string())
                    .collect::<Vec<_>>()
                    .join(","),
            ),
            Value::Null => None,
            _ => None,
        }
    }
}
