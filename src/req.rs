use anyhow::Result;
use http::Method;
use url::Url;

#[derive(Debug, Clone)]
pub struct RequestInit {
    pub method: Method,
    pub base: String,
    pub path: String,
    pub query: Vec<(String, Option<String>)>,
    pub header: Vec<(String, String)>,
    pub cookie: Vec<(String, String)>,
    pub body: Option<String>,
}

impl RequestInit {
    pub fn to_curl_args(&self) -> Result<Vec<String>, url::ParseError> {
        let mut args = vec![];

        args.push(format!("-X {}", self.method));
        let url: Url = self.clone().try_into()?;
        args.push(format!("'{}'", url));

        for (k, v) in self.header.iter() {
            args.push(format!("-H '{}: {}'", k, v));
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
            .filter_map(|(k, v)| {
                if let Some(v) = v {
                    match v.as_str() {
                        "true" | "false" => Some(k.to_string()),
                        _ => Some(format!("{}={}", k, v)),
                    }
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
