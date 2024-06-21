use url::Url;

use crate::req::{ParamsValue, RequestInit};

use super::{FormatError, RequestFormatter};

#[derive(Debug, Clone)]
pub(crate) struct CurlFormatter;

impl RequestFormatter for CurlFormatter {
    fn format(&self, req: &RequestInit) -> Result<std::string::String, FormatError> {
        let mut args = vec![];

        args.push(format!("-X {}", req.method));
        let url: Url = req.clone().try_into()?;
        args.push(format!("'{}'", url));

        for (k, v) in req.header.iter() {
            let v: ParamsValue = v.clone().into();
            args.push(format!("-H '{}: {}'", k, v));
        }

        if let Some(body) = &req.body {
            args.push(format!("-d '{}'", body));
        }

        Ok(args.join(" "))
    }
}
