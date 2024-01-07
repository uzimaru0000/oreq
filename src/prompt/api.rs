use anyhow::{anyhow, Result};
use color_eyre::owo_colors::{
    colors::css::{DarkGreen, White},
    OwoColorize,
};
use inquire::Select;
use openapiv3::{
    CookieStyle, HeaderStyle, OpenAPI, Parameter, ParameterData, ParameterSchemaOrContent,
    PathStyle, QueryStyle, RequestBody,
};
use reqwest::Method;
use std::{io::Write, vec};

use crate::{
    http::RequestInit,
    schema::{flat_schema, items, ReadSchema, ReferenceOrExt},
    serde::SerdeValue,
};

use super::{config::render_config, schema::SchemaPrompt, Prompt};

pub struct APIPrompt<'a> {
    api: &'a ReadSchema<OpenAPI>,
    base: &'a str,
    path: Option<String>,
    method: Option<Method>,
}

#[derive(Debug)]
enum Params {
    Query(String, Option<String>),
    Header(String, String),
    Path(String, String),
    _Cookie(String, String),
}

impl<'a> APIPrompt<'a> {
    pub fn new(
        api: &'a ReadSchema<OpenAPI>,
        base: &'a str,
        path: Option<String>,
        method: Option<Method>,
    ) -> Self {
        Self {
            api,
            base,
            path,
            method,
        }
    }

    pub fn prompt(&self) -> Result<RequestInit> {
        let path = self
            .api
            .schema
            .paths
            .iter()
            .map(|x| x.0)
            .collect::<Vec<_>>();
        let path = if let Some(path) = self.path.clone() {
            path
        } else {
            Select::new("Path", path)
                .with_render_config(render_config())
                .prompt()
                .map(|x| x.to_owned())?
        };

        let req = self
            .api
            .schema
            .paths
            .paths
            .get(&path)
            .ok_or(anyhow!("Path not found"))?;
        let item = req.item(&self.api.schema)?;

        let method = item.iter().map(|x| x.0.to_uppercase()).collect::<Vec<_>>();
        let method = if let Some(method) = self.method.clone() {
            method
        } else {
            let method = Select::new("Method", method)
                .with_render_config(render_config())
                .prompt()?;
            Method::from_bytes(method.as_bytes())?
        };

        let ope = match method {
            Method::GET => item.to_owned().get,
            Method::POST => item.to_owned().post,
            Method::PUT => item.to_owned().put,
            Method::DELETE => item.to_owned().delete,
            Method::OPTIONS => item.to_owned().options,
            Method::HEAD => item.to_owned().head,
            Method::PATCH => item.to_owned().patch,
            Method::TRACE => item.to_owned().trace,
            _ => unreachable!(),
        }
        .ok_or(anyhow!("Method not found"))?;

        let params = items(&ope.parameters, &self.api.schema).collect::<Result<Vec<_>>>()?;

        let params = params
            .into_iter()
            .map(|x| match x.to_owned() {
                Parameter::Query {
                    parameter_data,
                    allow_reserved,
                    style,
                    allow_empty_value,
                } => query_prompt(
                    &self.api.schema,
                    parameter_data,
                    allow_reserved,
                    style,
                    allow_empty_value,
                ),
                Parameter::Header {
                    parameter_data,
                    style,
                } => header_prompt(&self.api.schema, parameter_data, style),
                Parameter::Path {
                    parameter_data,
                    style,
                } => path_prompt(&self.api.schema, parameter_data, style),
                Parameter::Cookie {
                    parameter_data,
                    style,
                } => cookie_prompt(&self.api.schema, parameter_data, style),
            })
            .collect::<Result<Vec<_>>>()?;

        let req_body = if let Some(req_body) = ope.request_body {
            let req_body = req_body.item(&self.api.schema)?;
            let body = body_prompt(&self.api.schema, req_body)?;
            Some(body)
        } else {
            None
        };

        let mut req_init = RequestInit {
            base: self.base.to_owned(),
            method,
            path: path.to_owned(),
            query: vec![],
            header: if req_body.is_some() {
                vec![("Content-Type".to_owned(), "application/json".to_owned())]
            } else {
                vec![]
            },
            cookie: vec![],
            body: req_body,
        };

        params.iter().for_each(|x| {
            match x {
                Params::Query(name, value) => {
                    req_init.query.push((name.to_owned(), value.to_owned()));
                }
                Params::Header(name, value) => {
                    req_init.header.push((name.to_owned(), value.to_owned()));
                }
                Params::Path(name, value) => {
                    req_init.path = req_init.path.replace(&format!("{{{}}}", name), value);
                }
                Params::_Cookie(name, value) => {
                    req_init.cookie.push((name.to_owned(), value.to_owned()));
                }
            };
        });

        Ok(req_init)
    }
}

fn query_prompt(
    api: &OpenAPI,
    parameter_data: ParameterData,
    _allow_reserved: bool,
    _style: QueryStyle,
    allow_empty_value: Option<bool>,
) -> Result<Params> {
    let name = parameter_data.name;
    let is_required = parameter_data.required || !allow_empty_value.unwrap_or(true);
    let value = match parameter_data.format {
        ParameterSchemaOrContent::Schema(schema) => {
            let schema = schema.item(api)?;
            let (schema, is_req, description) = flat_schema(schema, api, is_required)?;
            let description = description.as_deref();
            let prompt = SchemaPrompt::new(&name, description, &schema, api);
            let schema = if is_req {
                let val = prompt.prompt()?;
                Some(val)
            } else {
                prompt.prompt_skippable()?
            };

            Ok(schema)
        }
        ParameterSchemaOrContent::Content(_) => Err(anyhow!("Content not supported")),
    }?;
    let value = value
        .map::<SerdeValue, _>(|x| x.into())
        .and_then(|x| x.to_query_string());

    Ok(Params::Query(name, value))
}

fn header_prompt(
    api: &OpenAPI,
    parameter_data: ParameterData,
    _style: HeaderStyle,
) -> Result<Params> {
    let name = parameter_data.name;
    let value = match parameter_data.format {
        ParameterSchemaOrContent::Schema(schema) => {
            let schema = schema.item(api)?;
            let (schema, is_req, description) = flat_schema(schema, api, parameter_data.required)?;
            let description = description.as_deref();
            let prompt = SchemaPrompt::new(&name, description, &schema, api);
            let schema = if is_req {
                let val = prompt.prompt()?;
                Some(val)
            } else {
                prompt.prompt_skippable()?
            };

            Ok(schema)
        }
        ParameterSchemaOrContent::Content(_) => Err(anyhow!("Content not supported")),
    }?;

    if let Some(value) = value {
        let value = serde_json::from_value(value)?;
        Ok(Params::Header(name, value))
    } else {
        Ok(Params::Header(name, "".to_owned()))
    }
}

fn path_prompt(api: &OpenAPI, parameter_data: ParameterData, _style: PathStyle) -> Result<Params> {
    let name = parameter_data.name;
    let value = match parameter_data.format {
        ParameterSchemaOrContent::Schema(schema) => {
            let schema = schema.item(api)?;
            let (schema, _, description) = flat_schema(schema, api, parameter_data.required)?;
            let description = description.as_deref();
            SchemaPrompt::new(&name, description, &schema, api).prompt()
        }
        ParameterSchemaOrContent::Content(_) => Err(anyhow!("Content not supported")),
    }?;
    let value = match value {
        serde_json::Value::Bool(b) => Ok(b.to_string()),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Null => Ok("".to_owned()),
        _ => serde_json::from_value(value),
    }?;

    Ok(Params::Path(name, value))
}

fn cookie_prompt(
    _api: &OpenAPI,
    _parameter_data: ParameterData,
    _style: CookieStyle,
) -> Result<Params> {
    todo!("cookie")
}

fn body_prompt(api: &OpenAPI, req_body: &RequestBody) -> Result<String> {
    writeln!(
        std::io::stderr(),
        "{}",
        " Input request body ".bg::<DarkGreen>().fg::<White>()
    )?;
    let req_body = req_body
        .content
        .get("application/json")
        .and_then(|x| x.schema.to_owned())
        .ok_or(anyhow!("Content not found"))?;

    let schema = req_body.item(api)?;
    let (schema, is_req, description) = flat_schema(schema, api, !schema.schema_data.nullable)?;
    let description = description.as_deref();
    let prompt = SchemaPrompt::new("Body", description, &schema, api);

    if is_req {
        let body = prompt.prompt()?;
        Ok(body.to_string())
    } else {
        let body = prompt.prompt_skippable()?;
        Ok(body.unwrap_or_default().to_string())
    }
}
