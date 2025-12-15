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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn get_temp_file_path() -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut path = std::env::temp_dir();
        path.push(format!("test_cache_{}.json", now));
        path
    }

    fn create_dummy_info() -> AssetClassInfo {
        AssetClassInfo {
            icon_url: Some("fWFc82js0fmoRAP-qOIPu5THSWqfSmTEL".to_string()),
            name: "Test Item".to_string(),
            market_hash_name: "Test Item".to_string(),
            market_name: "Test Item".to_string(),
            name_color: "FFFFFF".to_string(),
            type_: "Tool".to_string(),
        }
    }

    #[test]
    fn test_cache_operations() {
        let path = get_temp_file_path();

        // Ensure clean start
        if path.exists() {
            let _ = fs::remove_file(&path);
        }

        // Scope to ensure cache saves and variables are dropped before strict file check if needed (though save is explicit)
        {
            let cache = ItemCache::new(&path).expect("Failed to create cache");

            let info = create_dummy_info();

            // Test Insert
            cache
                .insert("100", "0", info.clone())
                .expect("Failed to insert");

            // Test Get
            let retrieved = cache.get("100", "0");
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap().name, "Test Item");
        } // cache dropped

        // Test Persistence (Load from disk)
        {
            let cache2 = ItemCache::new(&path).expect("Failed to load cache");
            let retrieved2 = cache2.get("100", "0");
            assert!(retrieved2.is_some());
            assert_eq!(retrieved2.unwrap().name, "Test Item");
        }

        // Cleanup
        let _ = fs::remove_file(&path);
    }
}
