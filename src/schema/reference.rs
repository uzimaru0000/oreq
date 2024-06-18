use indexmap::IndexMap;
use openapiv3::{OpenAPI, Parameter, PathItem, ReferenceOr, RequestBody, Response, Schema};
use serde::de::DeserializeOwned;

use crate::schema::error::SchemaError;

pub trait ReferenceOrExt<T>
where
    T: Lookup + DeserializeOwned + Clone,
{
    fn item<'a>(&'a self, api: &'a OpenAPI) -> Result<&'a T, SchemaError>;
}
pub trait Lookup: Sized {
    fn lookup(api: &OpenAPI) -> Option<&IndexMap<String, ReferenceOr<Self>>>;
}

impl<T> ReferenceOrExt<T> for openapiv3::ReferenceOr<T>
where
    T: Lookup + DeserializeOwned + Clone,
{
    fn item<'a>(&'a self, api: &'a OpenAPI) -> Result<&'a T, SchemaError> {
        match self {
            ReferenceOr::Item(item) => Ok(item),
            ReferenceOr::Reference { reference } => {
                if reference.starts_with("#/") {
                    let idx = reference.rfind('/').unwrap();
                    let key = &reference[idx + 1..];
                    let parameters = T::lookup(api)
                        .ok_or_else(|| SchemaError::ReferenceError(reference.to_owned()))?;
                    let parameters = parameters
                        .get(key)
                        .ok_or_else(|| SchemaError::ReferenceError(reference.to_owned()))?;
                    parameters.item(api)
                } else {
                    Err(SchemaError::UnsupportedExternalReference)
                }
            }
        }
    }
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
