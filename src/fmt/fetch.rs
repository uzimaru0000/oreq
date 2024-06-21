use indoc::formatdoc;
use url::Url;

use crate::req::{ParamsValue, RequestInit};

use super::{FormatError, RequestFormatter};

#[derive(Debug, Clone)]
pub(crate) struct FetchFormatter;

impl RequestFormatter for FetchFormatter {
    fn format(&self, req: &RequestInit) -> Result<String, FormatError> {
        let url: Url = req.clone().try_into()?;

        let method = req.method.to_uppercase();

        let headers = if req.header.is_empty() {
            None
        } else {
            let headers = req
                .header
                .iter()
                .map(|(k, v)| {
                    let v: ParamsValue = v.clone().into();
                    format!("'{}': '{}'", k, v)
                })
                .collect::<Vec<String>>()
                .join(",");
            Some(format!("{{{}}}", headers))
        };

        let body = req
            .body
            .clone()
            .map(|x| serde_json::to_string(&x))
            .transpose()?;

        Ok(formatdoc! {r#"
            fetch('{url}', {{
                method: '{method}',{headers}{body}
            }})
            "#,
            url = url,
            method = method,
            headers = headers.map(|x| format!("\n    headers: {},", x)).unwrap_or_default(),
            body = body.map(|x| format!("\n    body: '{}'", x)).unwrap_or_default()
        }
        .to_owned())
    }
}
