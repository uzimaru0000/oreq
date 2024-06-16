use anyhow::Result;
use serde_json::Value;
use url::Url;

use crate::serde::SerdeValue;

#[derive(Debug, Clone)]
pub enum Params {
    Query(String, Option<Value>),
    Header(String, Value),
    Path(String, Value),
    Cookie(String, Value),
}

#[derive(Debug, Clone)]
pub struct RequestInit {
    pub method: String,
    pub base: String,
    pub path: String,
    pub query: Vec<Params>,
    pub header: Vec<Params>,
    pub cookie: Vec<Params>,
    pub body: Option<Value>,
}

impl RequestInit {
    pub fn to_curl_args(&self) -> Result<Vec<String>, url::ParseError> {
        let mut args = vec![];

        args.push(format!("-X {}", self.method));
        let url: Url = self.clone().try_into()?;
        args.push(format!("'{}'", url));

        for header in self.header.iter() {
            if let Params::Header(k, v) = header {
                args.push(format!("-H '{}: {}'", k, v));
            }
        }

        if let Some(body) = &self.body {
            args.push(format!("-d '{}'", body));
        }

        Ok(args)
    }
}

impl TryInto<Url> for RequestInit {
    type Error = url::ParseError;

    fn try_into(self) -> Result<Url, Self::Error> {
        let mut url = Url::parse(&format!("{}{}", self.base, self.path))?;
        let query = self
            .query
            .iter()
            .filter_map(|query| {
                if let Params::Query(k, v) = query {
                    Some(
                        v.clone()
                            .map::<SerdeValue, _>(|x| x.into())
                            .and_then(|x| x.to_query_string())
                            .map(|x| format!("{}={}", k, x))
                            .unwrap_or(k.clone()),
                    )
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if !query.is_empty() {
            url.set_query(Some(&query.join("&")));
        }
        Ok(url)
    }
}
