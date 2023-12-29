use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::{env::current_dir, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use indexmap::IndexMap;
use inquire::Select;
use openapiv3::{
    ArrayType, BooleanType, IntegerType, NumberType, OpenAPI, Parameter, PathItem, ReferenceOr,
    RequestBody, Response, Schema, StringType, Type,
};
use serde::de::DeserializeOwned;

enum SupportExt {
    Json,
    Yaml,
}

#[derive(Debug, Clone)]
pub struct ReadSchema<T>
where
    T: DeserializeOwned,
{
    pub schema: T,
    pub base_dir: PathBuf,
}

impl<T> ReadSchema<T>
where
    T: DeserializeOwned,
{
    pub fn get_schema(path: PathBuf) -> Result<Self> {
        let path = &path.clone();
        let ext = path.extension().ok_or(anyhow!("No extension"))?;
        let ext = match ext.to_str() {
            Some("json") => Ok(SupportExt::Json),
            Some("yaml") => Ok(SupportExt::Yaml),
            Some("yml") => Ok(SupportExt::Yaml),
            _ => Err(anyhow!("Unsupported extension")),
        }?;

        let mut file = File::open(path)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;

        let result =
            match ext {
                SupportExt::Json => serde_json::from_slice::<T>(&content)
                    .with_context(|| "Parse failed".to_string()),
                SupportExt::Yaml => serde_yaml::from_slice::<T>(&content)
                    .with_context(|| "Parse failed".to_string()),
            }?;

        let base_dir = path.parent();
        let base_dir = if let Some(base_dir) = base_dir {
            base_dir.to_owned()
        } else {
            current_dir()?
        };

        Ok(Self {
            schema: result,
            base_dir,
        })
    }
}

pub(crate) trait ReferenceOrExt<T>
where
    T: Lookup + DeserializeOwned + Clone,
{
    fn item<'a>(&'a self, api: &'a OpenAPI) -> Result<&'a T>;
}
pub(crate) trait Lookup: Sized {
    fn lookup(api: &OpenAPI) -> Option<&IndexMap<String, ReferenceOr<Self>>>;
}

impl<T> ReferenceOrExt<T> for openapiv3::ReferenceOr<T>
where
    T: Lookup + DeserializeOwned + Clone,
{
    fn item<'a>(&'a self, api: &'a OpenAPI) -> Result<&'a T> {
        match self {
            ReferenceOr::Item(item) => Ok(item),
            ReferenceOr::Reference { reference } => {
                if reference.starts_with("#/") {
                    let idx = reference.rfind('/').unwrap();
                    let key = &reference[idx + 1..];
                    let parameters = T::lookup(api).with_context(|| anyhow!("No parameters"))?;
                    return parameters
                        .get(key)
                        .unwrap_or_else(|| panic!("key {} is missing", key))
                        .item(api);
                } else {
                    Err(anyhow!(
                        "Unsupported external reference. Please bundle your schema"
                    ))
                }
            }
        }
    }
}

pub(crate) fn items<'a, T>(
    refs: &'a [ReferenceOr<T>],
    api: &'a OpenAPI,
) -> impl Iterator<Item = Result<&'a T>>
where
    T: Lookup + DeserializeOwned + Clone,
{
    refs.iter().map(|x| x.item(api))
}

impl Lookup for Parameter {
    fn lookup(api: &OpenAPI) -> Option<&IndexMap<String, ReferenceOr<Self>>> {
        api.components.as_ref().map(|x| &x.parameters)
    }
}

impl Lookup for RequestBody {
    fn lookup(api: &OpenAPI) -> Option<&IndexMap<String, ReferenceOr<Self>>> {
        api.components.as_ref().map(|x| &x.request_bodies)
    }
}

impl Lookup for Response {
    fn lookup(api: &OpenAPI) -> Option<&IndexMap<String, ReferenceOr<Self>>> {
        api.components.as_ref().map(|x| &x.responses)
    }
}

impl Lookup for Schema {
    fn lookup(api: &OpenAPI) -> Option<&IndexMap<String, ReferenceOr<Self>>> {
        api.components.as_ref().map(|x| &x.schemas)
    }
}

impl Lookup for PathItem {
    fn lookup(api: &OpenAPI) -> Option<&IndexMap<String, ReferenceOr<Self>>> {
        Some(&api.paths.paths)
    }
}

pub enum SchemaType {
    Object(IndexMap<String, (SchemaType, bool)>),
    Array(ArrayType),
    String(StringType),
    Number(NumberType),
    Integer(IntegerType),
    Boolean(BooleanType),
}

impl Display for SchemaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaType::Object(obj) => {
                let mut s = String::new();
                for (k, v) in obj {
                    let (v, is_required) = v;
                    let v = format!("{}", v);
                    if *is_required {
                        s.push_str(&format!("    {}: {},\n", k, v));
                    } else {
                        s.push_str(&format!("    {}: Option<{}>,\n", k, v));
                    }
                }

                write!(f, "{{\n{}  }}", s)
            }
            SchemaType::Array(t) => write!(
                f,
                "{}",
                serde_json::to_string(t).map_err(|_| std::fmt::Error)?,
            ),
            SchemaType::String(_) => write!(f, "String",),
            SchemaType::Number(_) => write!(f, "Number",),
            SchemaType::Integer(_) => write!(f, "Integer",),
            SchemaType::Boolean(_) => write!(f, "Boolean",),
        }
    }
}

pub fn flat_schema(
    schema: &Schema,
    api: &OpenAPI,
    is_required: bool,
) -> Result<(SchemaType, bool)> {
    match &schema.schema_kind {
        openapiv3::SchemaKind::Type(types) => match types {
            Type::Object(object) => {
                let properties = object.to_owned().properties;
                let obj = properties
                    .into_iter()
                    .map(|(k, v)| {
                        let is_required = object.required.contains(&k);
                        let v = v.unbox();
                        let v = v.item(api)?;
                        let v = flat_schema(v, api, is_required)?;
                        Ok((k, v))
                    })
                    .collect::<Result<IndexMap<_, _>>>()?;

                Ok((SchemaType::Object(obj), is_required))
            }
            Type::Array(array) => Ok((SchemaType::Array(array.to_owned()), is_required)),
            Type::String(t) => Ok((SchemaType::String(t.to_owned()), is_required)),
            Type::Number(t) => Ok((SchemaType::Number(t.to_owned()), is_required)),
            Type::Integer(t) => Ok((SchemaType::Integer(t.to_owned()), is_required)),
            Type::Boolean(t) => Ok((SchemaType::Boolean(t.to_owned()), is_required)),
        },
        openapiv3::SchemaKind::OneOf { one_of } => {
            let one_of = one_of
                .iter()
                .map(|x| {
                    let x = x.item(api)?;
                    let (x, _) = flat_schema(x, api, is_required)?;
                    Ok(x)
                })
                .collect::<Result<Vec<_>>>()?;
            let select = Select::new("Select one of schema", one_of).prompt()?;

            Ok((select, is_required))
        }
        openapiv3::SchemaKind::AllOf { .. } => todo!("AllOf"),
        openapiv3::SchemaKind::AnyOf { .. } => todo!("AnyOf"),
        openapiv3::SchemaKind::Not { .. } => todo!("Not"),
        openapiv3::SchemaKind::Any(_) => todo!("Any"),
    }
}

#[cfg(test)]
mod tests {
    use super::ReadSchema;
    use openapiv3::{OpenAPI, PathItem, Type};
    use std::path::PathBuf;

    #[test]
    fn test_read_schema() {
        let path = PathBuf::from("tests/fixtures/duck.yaml");
        let schema = ReadSchema::<OpenAPI>::get_schema(path).unwrap();
        assert_eq!(schema.schema.openapi, "3.0.0");
    }

    #[test]
    fn test_read_part_of_schema() {
        let path = PathBuf::from("tests/fixtures/types.yaml");
        let schema = ReadSchema::<Type>::get_schema(path).unwrap();

        if let Type::Object(_) = schema.schema {
        } else {
            unreachable!()
        }
    }

    #[test]
    fn test_read_path_item() {
        let path = PathBuf::from("tests/fixtures/path_item.yaml");
        let schema = ReadSchema::<PathItem>::get_schema(path).unwrap();

        assert!(schema.schema.get.is_some());
    }
}
