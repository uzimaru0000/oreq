use std::fmt::Display;

use anyhow::Result;
use color_eyre::owo_colors::{
    colors::css::{Black, LightGreen, Red, White},
    OwoColorize,
};
use reqwest::{IntoUrl, Method, Url};

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
    pub fn to_curl_args(&self) -> Result<Vec<String>> {
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
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Url> {
        let mut url = Url::parse(&format!("{}{}", self.base, self.path))?;
        let query = self
            .query
            .iter()
            .filter_map(|(k, v)| {
                if let Some(v) = v {
                    match v.as_str() {
                        "true" | "false" => Some(format!("{}", k)),
                        _ => Some(format!("{}={}", k, v)),
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if query.len() > 0 {
            url.set_query(Some(&query.join("&")));
        }
        Ok(url)
    }
}

pub struct Response<T: IntoUrl + Clone> {
    pub url: T,
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl<T: IntoUrl + Clone> Display for Response<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.status {
            200..=299 => write!(f, "{}", self.status.bg::<LightGreen>().fg::<White>()),
            400..=499 | 500..=599 => write!(f, "{}", self.status.bg::<Red>().fg::<White>()),
            _ => write!(f, "{}", self.status.bg::<White>().fg::<Black>()),
        }?;

        let url = self.url.clone().into_url().map_err(|_| std::fmt::Error)?;
        writeln!(f, " {}", url)?;

        for (k, v) in &self.headers {
            writeln!(f, "{}: {}", k, v)?;
        }

        writeln!(f, "{}", self.body)
    }
}
