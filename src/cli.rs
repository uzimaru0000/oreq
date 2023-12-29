use anyhow::anyhow;
use std::{error::Error, path::PathBuf};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    pub schema: PathBuf,
    #[arg(long, short)]
    pub base_url: Option<String>,
    #[arg(long, short = 'H', value_parser = parse_key_val::<String, String>)]
    pub headers: Option<Vec<(String, String)>>,
}

fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let mut split = s.split(":");
    let key = split.next().ok_or(anyhow!("No key"))?;
    let key = key.trim();
    let value = split.next().ok_or(anyhow!("No value"))?;
    let value = value.trim();

    Ok((key.parse()?, value.parse()?))
}
