use anyhow::{anyhow, Result};
use http::Method;
use openapiv3::OpenAPI;
use promptuity::{themes::FancyTheme, Term};
use serde_json::{json, Value};
use std::error::Error;

use clap::Parser;

use crate::{
    error::AppError,
    fmt::{Formatter, RequestFormatter},
    prompt::Prompt,
};
use oreq::schema::read::ReadSchema;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(help = "OpenAPI schema path", value_hint = clap::ValueHint::FilePath)]
    pub schema: String,
    #[arg(long, short, help = "Base URL", value_hint = clap::ValueHint::Url)]
    pub base_url: Option<String>,
    #[arg(long, short = 'H', value_parser = parse_key_val)]
    pub headers: Option<Vec<(String, serde_json::Value)>>,
    #[arg(long, short, help = "Path to request")]
    pub path: Option<String>,
    #[arg(long = "request", short = 'X', help = "Method to use")]
    pub method: Option<Method>,
    #[arg(long = "param", short = 'P', help = "Path parameters", value_parser = parse_body)]
    pub path_param: Option<Vec<(String, serde_json::Value)>>,
    #[arg(long, short, help = "Query parameters", value_parser = parse_body)]
    pub query_param: Option<Vec<(String, serde_json::Value)>>,
    #[arg(long, short, help = "Request body", value_parser = parse_body)]
    pub field: Option<Vec<(String, serde_json::Value)>>,
    #[arg(long = "format", help = "Output format", default_value = "curl")]
    pub fmt: Formatter,
}

fn parse_key_val(
    s: &str,
) -> Result<(String, serde_json::Value), Box<dyn Error + Send + Sync + 'static>> {
    let (key, value) = s.split_once(':').ok_or(anyhow!("Invalid format"))?;
    let key = key.trim();
    let value = value.trim();

    let key = key.to_string();
    let value = value.parse::<Value>().unwrap_or_else(|_| json!(value));

    Ok((key, value))
}

fn parse_body(
    s: &str,
) -> Result<(String, serde_json::Value), Box<dyn Error + Send + Sync + 'static>> {
    let (key, value) = s.split_once('=').ok_or(anyhow!("Invalid format"))?;
    let key = key.trim();
    let value = value.trim();

    let key = key.to_string();
    let value = value.parse::<Value>().unwrap_or_else(|_| json!(value));

    Ok((key, value))
}

impl Cli {
    pub fn run(&self) -> Result<(), AppError> {
        let api = if self.schema == "-" {
            ReadSchema::<OpenAPI>::get_schema_from_stdin()
        } else {
            ReadSchema::<OpenAPI>::get_schema(self.schema.clone().into())
        }
        .map_err(|_| AppError::SchemaParseError)?;
        let server = self
            .base_url
            .clone()
            .or(api.schema.servers.first().map(|x| x.url.clone()))
            .ok_or(AppError::NoServers)?;

        let mut term = Term::default();
        let mut theme = FancyTheme::default();
        let mut init = Prompt::new(api.schema, &mut term, &mut theme).run(
            self.path.clone(),
            self.method.clone(),
            self.path_param
                .clone()
                .map(|x| x.into_iter().collect())
                .unwrap_or_default(),
            self.query_param
                .clone()
                .map(|x| x.into_iter().collect())
                .unwrap_or_default(),
            self.headers
                .clone()
                .map(|x| x.into_iter().collect())
                .unwrap_or_default(),
            self.field
                .clone()
                .map(|x| x.into_iter().collect())
                .unwrap_or_default(),
        )?;
        init.base = server;
        if let Some(headers) = self.headers.clone() {
            init.header.extend(headers);
        }
        let fmt: Box<dyn RequestFormatter> = self.fmt.clone().into();
        let out = fmt.format(&init)?;

        eprintln!();
        println!("{}", out);

        Ok(())
    }
}
