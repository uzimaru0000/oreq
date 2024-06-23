use anyhow::anyhow;
use http::Method;
use indexmap::IndexMap;
use openapiv3::{OpenAPI, Operation, Parameter, ParameterData, ParameterSchemaOrContent, PathItem};
use promptuity::{prompts::SelectOption, Promptuity, Terminal, Theme};

use oreq::{
    prompts::{enumeration::Enumeration, optional_prompt_builder, prompt_builder},
    schema::{error::SchemaError, reference::ReferenceOrExt},
};
use serde_json::Value;

use crate::{
    error::AppError,
    req::{ParamsValue, RequestInit},
};

struct ParamsMap<T> {
    query: Vec<T>,
    header: Vec<T>,
    path: Vec<T>,
    cookie: Vec<T>,
}
impl<T> Default for ParamsMap<T> {
    fn default() -> Self {
        Self {
            query: Vec::new(),
            header: Vec::new(),
            path: Vec::new(),
            cookie: Vec::new(),
        }
    }
}
impl<T> From<IndexMap<String, Vec<T>>> for ParamsMap<T>
where
    T: Clone,
{
    fn from(map: IndexMap<String, Vec<T>>) -> Self {
        Self {
            query: map.get("query").cloned().unwrap_or_default(),
            header: map.get("header").cloned().unwrap_or_default(),
            path: map.get("path").cloned().unwrap_or_default(),
            cookie: map.get("cookie").cloned().unwrap_or_default(),
        }
    }
}

pub struct Prompt<'a, W>
where
    W: std::io::Write,
{
    api: OpenAPI,
    provider: Promptuity<'a, W>,
}

impl<'a, W> Prompt<'a, W>
where
    W: std::io::Write,
{
    pub fn new(api: OpenAPI, term: &'a mut dyn Terminal<W>, theme: &'a mut dyn Theme<W>) -> Self {
        Self {
            api,
            provider: Promptuity::new(term, theme),
        }
    }

    pub fn run(
        &mut self,
        path: Option<String>,
        method: Option<Method>,
        path_params: IndexMap<String, Value>,
        query_params: IndexMap<String, Value>,
        header: IndexMap<String, Value>,
        fields: IndexMap<String, Value>,
    ) -> Result<RequestInit, AppError> {
        self.provider.term().clear()?;

        self.provider.with_intro("Build Request").begin()?;
        let mut path_prompt = self.path_prompt()?;
        let (path, path_item) = if let Some(path) = path {
            let path_item = self
                .api
                .paths
                .paths
                .get(&path)
                .ok_or_else(|| anyhow!("Path not found"))?;
            let path_item = path_item.item(&self.api)?;
            (path, path_item.clone())
        } else {
            self.provider.prompt(&mut path_prompt)?
        };
        let mut method_prompt = self.method_prompt(&path_item)?;
        let (method, operation) = if let Some(method) = method {
            match method {
                Method::GET => {
                    let operation = path_item
                        .get
                        .clone()
                        .ok_or_else(|| anyhow!("Method not found"))?;
                    ("GET".to_owned(), operation)
                }
                Method::POST => {
                    let operation = path_item
                        .post
                        .clone()
                        .ok_or_else(|| anyhow!("Method not found"))?;
                    ("POST".to_owned(), operation)
                }
                Method::PUT => {
                    let operation = path_item
                        .put
                        .clone()
                        .ok_or_else(|| anyhow!("Method not found"))?;
                    ("PUT".to_owned(), operation)
                }
                Method::DELETE => {
                    let operation = path_item
                        .delete
                        .clone()
                        .ok_or_else(|| anyhow!("Method not found"))?;
                    ("DELETE".to_owned(), operation)
                }
                Method::PATCH => {
                    let operation = path_item
                        .patch
                        .clone()
                        .ok_or_else(|| anyhow!("Method not found"))?;
                    ("PATCH".to_owned(), operation)
                }
                _ => self.provider.prompt(&mut method_prompt)?,
            }
        } else {
            self.provider.prompt(&mut method_prompt)?
        };

        let mut params_data = ParamsMap::default();
        for param in operation.parameters {
            let param = param.item(&self.api)?;

            match param {
                Parameter::Query { parameter_data, .. } => {
                    params_data.query.push(parameter_data.clone())
                }
                Parameter::Header { parameter_data, .. } => {
                    params_data.header.push(parameter_data.clone())
                }
                Parameter::Path { parameter_data, .. } => {
                    params_data.path.push(parameter_data.clone())
                }
                Parameter::Cookie { parameter_data, .. } => {
                    params_data.cookie.push(parameter_data.clone())
                }
            }
        }

        let mut params = ParamsMap::default();
        let prompts = vec![
            (
                &mut params.path,
                &params_data.path,
                path_params,
                "Path Parameters",
            ),
            (
                &mut params.query,
                &params_data.query,
                query_params,
                "Query Parameters",
            ),
            (
                &mut params.header,
                &params_data.header,
                header,
                "Header Parameters",
            ),
            (
                &mut params.cookie,
                &params_data.cookie,
                IndexMap::new(),
                "Cookie Parameters",
            ),
        ];

        for (map, data, cli_input, msg) in prompts {
            if data.is_empty() {
                continue;
            }

            self.provider.step(msg)?;
            for param in data {
                let value = if param.required {
                    let mut prompt = self.parameter_prompt(param)?;
                    if let Some(value) = cli_input.get(&param.name) {
                        Some(value.clone())
                    } else {
                        Some(self.provider.prompt(&mut *prompt)?)
                    }
                } else {
                    let mut prompt = self.optional_parameter_prompt(param)?;
                    if let Some(value) = cli_input.get(&param.name) {
                        Some(value.clone())
                    } else {
                        self.provider.prompt(&mut *prompt)?
                    }
                };

                map.push((param.name.to_owned(), value));
            }
        }

        let req_body = if let Some(req_body) = operation.request_body {
            let req_body = req_body.item(&self.api)?;
            let req_body = req_body
                .content
                .get("application/json")
                .and_then(|x| x.schema.to_owned())
                .ok_or_else(|| anyhow!("Only supported 'application/json'"))?;
            let req_body = req_body.item(&self.api)?;

            let mut prompt = prompt_builder(
                &self.api,
                req_body,
                "Request Body".to_owned(),
                req_body.schema_data.description.clone(),
                Some(fields),
            );
            let value = self.provider.prompt(&mut *prompt)?;
            Some(value)
        } else {
            None
        };

        self.provider.finish()?;

        Ok(RequestInit {
            method,
            base: String::new(),
            path: params
                .path
                .into_iter()
                .filter_map(|(k, v)| v.map(|v| (k.to_owned(), ParamsValue::from(v))))
                .fold(path, |acc, (name, value)| {
                    acc.replace(&format!("{{{}}}", name), &value.to_string())
                }),
            query: params.query,
            header: params
                .header
                .into_iter()
                .filter_map(|(k, v)| v.map(|v| (k.to_owned(), v)))
                .collect(),
            cookie: params
                .cookie
                .into_iter()
                .filter_map(|(k, v)| v.map(|v| (k.to_owned(), v)))
                .collect(),
            body: req_body,
        })
    }

    fn path_prompt(&self) -> Result<Enumeration<(String, PathItem)>, SchemaError> {
        let mut paths = IndexMap::new();

        for (path, path_item) in self.api.paths.clone() {
            let item = path_item.item(&self.api)?;
            paths.insert(path, item.clone());
        }

        let options = paths
            .into_iter()
            .map(|(path, item)| {
                let opts = SelectOption::new(path.to_owned(), (path.to_owned(), item.clone()));
                if let Some(description) = item.description {
                    opts.with_hint(description)
                } else {
                    opts
                }
            })
            .collect();

        Ok(Enumeration::new("Path".to_owned(), options))
    }

    fn method_prompt(
        &self,
        path_item: &PathItem,
    ) -> Result<Enumeration<(String, Operation)>, SchemaError> {
        let options = vec![
            ("GET", path_item.get.clone()),
            ("POST", path_item.post.clone()),
            ("PUT", path_item.put.clone()),
            ("DELETE", path_item.delete.clone()),
            ("PATCH", path_item.patch.clone()),
        ]
        .into_iter()
        .filter_map(|(k, x)| x.map(|v| (k, v)))
        .map(|(k, v)| SelectOption::new(k.to_owned(), (k.to_owned(), v.clone())))
        .collect::<Vec<_>>();

        Ok(Enumeration::new("Method".to_owned(), options))
    }

    fn parameter_prompt(
        &self,
        parameter: &ParameterData,
    ) -> Result<Box<dyn promptuity::Prompt<Output = Value>>, SchemaError> {
        match parameter.format.clone() {
            ParameterSchemaOrContent::Schema(schema) => {
                let item = schema.item(&self.api)?;
                Ok(prompt_builder(
                    &self.api,
                    item,
                    parameter.name.clone(),
                    parameter.description.clone(),
                    None,
                ))
            }
            ParameterSchemaOrContent::Content(_) => Err(SchemaError::UnsupportedSchema),
        }
    }

    fn optional_parameter_prompt(
        &self,
        parameter: &ParameterData,
    ) -> Result<Box<dyn promptuity::Prompt<Output = Option<Value>>>, SchemaError> {
        match parameter.format.clone() {
            ParameterSchemaOrContent::Schema(schema) => {
                let item = schema.item(&self.api)?;
                Ok(optional_prompt_builder(
                    &self.api,
                    item,
                    parameter.name.clone(),
                    parameter.description.clone(),
                    None,
                ))
            }
            ParameterSchemaOrContent::Content(_) => Err(SchemaError::UnsupportedSchema),
        }
    }
}
