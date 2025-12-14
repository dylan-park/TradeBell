use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use crate::models::AssetClassInfo;

#[derive(Clone)]
pub struct ItemCache {
    data: Arc<RwLock<HashMap<String, AssetClassInfo>>>,
    file_path: PathBuf,
}

impl ItemCache {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file_path = path.as_ref().to_path_buf();
        let mut data = HashMap::new();

        if file_path.exists() {
            let content = fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read cache file: {:?}", file_path))?;
            if !content.is_empty() {
                data = serde_json::from_str(&content)
                    .with_context(|| format!("Failed to parse cache file: {:?}", file_path))?;
            }
        }

        Ok(Self {
            data: Arc::new(RwLock::new(data)),
            file_path,
        })
    }

    pub fn get(&self, classid: &str, _instanceid: &str) -> Option<AssetClassInfo> {
        let key = classid.to_string();
        let data = self.data.read().ok()?;
        data.get(&key).cloned()
    }

    pub fn insert(&self, classid: &str, _instanceid: &str, info: AssetClassInfo) -> Result<()> {
        let key = classid.to_string();

        {
            let mut data = self
                .data
                .write()
                .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            data.insert(key, info);
        }

        self.save()?;
        Ok(())
    }

    fn save(&self) -> Result<()> {
        let data = self
            .data
            .read()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        let content = serde_json::to_string_pretty(&*data)?;
        fs::write(&self.file_path, content)
            .with_context(|| format!("Failed to write cache file: {:?}", self.file_path))?;
        Ok(())
    }
}
