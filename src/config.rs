use anyhow::Result;
use config::{Config, File};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub log_level: Option<String>, 
    pub program: HashMap<String, ProgramConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProgramConfig {
    pub command: String,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default)]
    pub restart: Option<String>,
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
}

pub fn load_config(path: &str) -> Result<GlobalConfig> {
    let settings = Config::builder()
        .add_source(File::with_name(path))
        .build()?;

    let config: GlobalConfig = settings.try_deserialize()?;
    Ok(config)
}
