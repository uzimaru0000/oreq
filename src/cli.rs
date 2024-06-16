use anyhow::{anyhow, Result};
use http::Method;
use openapiv3::OpenAPI;
use promptuity::{themes::FancyTheme, Term};
use serde_json::json;
use std::{error::Error, path::PathBuf};

use clap::Parser;

use crate::{error::AppError, prompt::Prompt};
use oreq::schema::read::ReadSchema;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(help = "OpenAPI schema path", value_hint = clap::ValueHint::FilePath)]
    pub schema: PathBuf,
    #[arg(long, short, help = "Base URL", value_hint = clap::ValueHint::Url)]
    pub base_url: Option<String>,
    #[arg(long, short = 'H', value_parser = parse_key_val::<String, String>)]
    pub headers: Option<Vec<(String, String)>>,
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
}

fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let mut split = s.split(':');
    let key = split.next().ok_or(anyhow!("No key"))?;
    let key = key.trim();
    let value = split.next().ok_or(anyhow!("No value"))?;
    let value = value.trim();

    Ok((key.parse()?, value.parse()?))
}

fn parse_body(
    s: &str,
) -> Result<(String, serde_json::Value), Box<dyn Error + Send + Sync + 'static>> {
    let (key, value) = s.split_once('=').ok_or(anyhow!("Invalid format"))?;
    let key = key.trim();
    let value = value.trim();

    Ok((key.to_string(), json!(value)))
}

impl Cli {
    pub fn run(&self) -> Result<(), AppError> {
        let api = ReadSchema::<OpenAPI>::get_schema(self.schema.clone())
            .map_err(|_| AppError::SchemaError)?;
        let server = self
            .base_url
            .clone()
            .or(api.schema.servers.first().map(|x| x.url.clone()))
            .ok_or(AppError::NoServers)?;

        let mut term = Term::default();
        let mut theme = FancyTheme::default();
        let mut init = Prompt::new(api.schema, &mut term, &mut theme)
            .run()
            .map_err(AppError::PromptError)?;
        init.base = server;
        let args = init.to_curl_args().map_err(AppError::ParseError)?;

        println!("{}", args.join(" "));

        Ok(())
    }
}
