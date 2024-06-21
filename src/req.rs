use std::fmt::Display;

use anyhow::Result;
use serde_json::Value;
use url::Url;

pub struct ParamsValue(Value);
impl From<Value> for ParamsValue {
    fn from(value: Value) -> Self {
        Self(value)
    }
}

impl Display for ParamsValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match &self.0 {
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.to_owned(),
            Value::Array(a) => a
                .iter()
                .map::<ParamsValue, _>(|x| x.clone().into())
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(","),
            Value::Null => String::new(),
            _ => String::new(),
        };

        write!(f, "{}", string)
    }
}

#[derive(Debug, Clone)]
pub struct RequestInit {
    pub method: String,
    pub base: String,
    pub path: String,
    pub query: Vec<(String, Option<Value>)>,
    pub header: Vec<(String, Value)>,
    pub cookie: Vec<(String, Value)>,
    pub body: Option<Value>,
}

impl TryInto<Url> for RequestInit {
    type Error = url::ParseError;

    fn try_into(self) -> Result<Url, Self::Error> {
        let mut url = Url::parse(&format!("{}{}", self.base, self.path))?;
        let query = self
            .query
            .into_iter()
            .filter_map(|(k, v)| v.map(|v| (k.clone(), v)))
            .filter_map(|(k, v)| match v {
                Value::Bool(true) => Some(k.to_string()),
                Value::Bool(false) => None,
                _ => Some(format!("{}={}", k, ParamsValue(v))),
            })
            .collect::<Vec<_>>();

        if !query.is_empty() {
            url.set_query(Some(&query.join("&")));
        }
        Ok(url)
    }
}
