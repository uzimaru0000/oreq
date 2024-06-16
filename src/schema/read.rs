use std::{
    env::current_dir,
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use anyhow::{anyhow, Context as _};
use crossterm::tty::IsTty;
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
    pub fn get_schema(path: PathBuf) -> anyhow::Result<Self> {
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

    pub fn get_schema_from_stdin() -> anyhow::Result<Self> {
        let mut content = Vec::new();
        if !io::stdin().is_tty() {
            io::stdin().read_to_end(&mut content)?;
        }

        let yaml_result = serde_yaml::from_slice::<T>(&content);
        let json_result = serde_json::from_slice::<T>(&content);

        let result = yaml_result
            .or(json_result)
            .with_context(|| "Parse failed".to_string())?;

        Ok(Self {
            schema: result,
            base_dir: current_dir()?,
        })
    }
}
