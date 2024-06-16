use anyhow::anyhow;
use indexmap::IndexMap;
use openapiv3::{OpenAPI, Operation, Parameter, ParameterData, ParameterSchemaOrContent, PathItem};
use promptuity::{prompts::SelectOption, Promptuity, Terminal, Theme};

use oreq::{
    prompts::{enumeration::Enumeration, prompt_builder},
    schema::reference::ReferenceOrExt,
};
use serde_json::Value;

use crate::req::{Params, RequestInit};

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

    pub fn run(&mut self) -> anyhow::Result<RequestInit> {
        self.provider.term().clear()?;

        self.provider.begin()?;
        let mut path = self.path_prompt()?;
        let (path, path_item) = self.provider.prompt(&mut path)?;
        let mut method = self.method_prompt(&path_item)?;
        let (method, operation) = self.provider.prompt(&mut method)?;

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
        if !params_data.path.is_empty() {
            self.provider.step("Path Parameters")?;
            for param in &params_data.path {
                let mut prompt = self.parameter_prompt(param)?;
                let value = self.provider.prompt(&mut *prompt)?;
                params.path.push(Params::Path(param.name.to_owned(), value));
            }
        }

        if !params_data.query.is_empty() {
            self.provider.step("Query Parameters")?;
            for param in &params_data.query {
                let mut prompt = self.parameter_prompt(param)?;
                let value = self.provider.prompt(&mut *prompt)?;
                params
                    .query
                    .push(Params::Query(param.name.to_owned(), Some(value)));
            }
        }

        if !params_data.header.is_empty() {
            self.provider.step("Header Parameters")?;
            for param in &params_data.header {
                let mut prompt = self.parameter_prompt(param)?;
                let value = self.provider.prompt(&mut *prompt)?;
                params
                    .header
                    .push(Params::Header(param.name.to_owned(), value));
            }
        }

        if !params_data.cookie.is_empty() {
            self.provider.step("Cookie Parameters")?;
            for param in &params_data.cookie {
                let mut prompt = self.parameter_prompt(param)?;
                let value = self.provider.prompt(&mut *prompt)?;
                params
                    .cookie
                    .push(Params::Cookie(param.name.to_owned(), value));
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

            let mut prompt = prompt_builder(&self.api, req_body, "Request Body".to_owned());
            let value = self.provider.prompt(&mut *prompt)?;
            Some(value)
        } else {
            None
        };

        Ok(RequestInit {
            method,
            base: String::new(),
            path: params.path.iter().fold(path, |acc, x| {
                if let Params::Path(name, value) = x {
                    let value = match value {
                        Value::Bool(b) => b.to_string(),
                        Value::Number(n) => n.to_string(),
                        Value::String(s) => s.to_owned(),
                        Value::Null => "".to_owned(),
                        _ => "".to_owned(),
                    };

                    acc.replace(&format!("{{{}}}", name), &value.to_string())
                } else {
                    acc
                }
            }),
            query: params.query,
            header: params.header,
            cookie: params.cookie,
            body: req_body,
        })
    }

    fn path_prompt(&self) -> anyhow::Result<Enumeration<(String, PathItem)>> {
        let mut paths = IndexMap::new();

        for (path, path_item) in self.api.paths.clone() {
            let item = path_item.item(&self.api)?;
            paths.insert(path, item.clone());
        }

        let options = paths
            .into_iter()
            .map(|(path, item)| SelectOption::new(path.to_owned(), (path.to_owned(), item.clone())))
            .collect();

        Ok(Enumeration::new("Path".to_owned(), options))
    }

    fn method_prompt(
        &self,
        path_item: &PathItem,
    ) -> anyhow::Result<Enumeration<(String, Operation)>> {
        let options = vec![
            ("GET", path_item.get.clone()),
            ("POST", path_item.post.clone()),
            ("PUT", path_item.put.clone()),
            ("DELETE", path_item.delete.clone()),
            ("PATCH", path_item.patch.clone()),
        ]
        .into_iter()
        .filter_map(|(k, x)| x.map(|v| (k, v)))
        .map(|(k, v)| SelectOption::new(k.to_owned(), (k.to_owned(), v)))
        .collect::<Vec<_>>();

        Ok(Enumeration::new("Method".to_owned(), options))
    }

    fn parameter_prompt(
        &self,
        parameter: &ParameterData,
    ) -> anyhow::Result<Box<dyn promptuity::Prompt<Output = Value>>> {
        match parameter.format.clone() {
            ParameterSchemaOrContent::Schema(schema) => {
                let item = schema.item(&self.api)?;
                Ok(prompt_builder(&self.api, item, parameter.name.clone()))
            }
            ParameterSchemaOrContent::Content(_) => Err(anyhow!("Content not supported")),
        }
    }
}
