use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub telegram_token: String,
    pub telegram_chat_id: String,
    pub polling_interval_seconds: u64,
    pub accounts: Vec<AccountConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AccountConfig {
    pub name: String,
    pub api_key: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Path::new("config.json");

        if !config_path.exists() {
            anyhow::bail!("config.json not found. Please create one based on the documentation.");
        }

        let content = fs::read_to_string(config_path).context("Failed to read config.json")?;

        let config: Config =
            serde_json::from_str(&content).context("Failed to parse config.json")?;

        Ok(config)
    }
}
