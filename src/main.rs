use crate::prompt::api::APIPrompt;
use anyhow::{Context, Result};
use clap::Parser;
use openapiv3::OpenAPI;

mod cli;
mod http;
mod prompt;
mod schema;
mod serde;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    let api = schema::ReadSchema::<OpenAPI>::get_schema(cli.schema)?;
    let server = cli
        .base_url
        .or(api.schema.servers.first().map(|x| x.url.clone()))
        .with_context(|| "No servers in schema")?;

    let mut init = APIPrompt::new(&api, &server, cli.path, cli.method).prompt()?;
    if let Some(from_cli) = cli.headers {
        init.header = [init.header, from_cli].concat();
    }
    let args = init.to_curl_args()?;

    println!("{}", args.join(" "));

    Ok(())
}
