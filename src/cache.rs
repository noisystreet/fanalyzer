use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing;

pub struct FundCache {
    cache_path: PathBuf,
    code_to_name: HashMap<String, String>,
    name_to_code: HashMap<String, String>,
}

impl Default for FundCache {
    fn default() -> Self {
        Self::new()
    }
}

impl FundCache {
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("fanalyzer");
        Self::with_root(cache_dir)
    }

    pub fn with_root(cache_dir: PathBuf) -> Self {
        let cache_path = cache_dir.join("fund_names.json");

        let code_to_name: HashMap<String, String> = if cache_path.exists() {
            match fs::read_to_string(&cache_path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to read cache file");
                    HashMap::new()
                }
            }
        } else {
            HashMap::new()
        };

        let name_to_code: HashMap<String, String> = code_to_name
            .iter()
            .map(|(k, v)| (v.clone(), k.clone()))
            .collect();

        Self {
            cache_path,
            code_to_name,
            name_to_code,
        }
    }

    pub fn get_name(&self, code: &str) -> Option<String> {
        self.code_to_name.get(code).cloned()
    }

    pub fn get_code(&self, name: &str) -> Option<String> {
        self.name_to_code.get(name).cloned()
    }

    pub fn set_mapping(&mut self, code: &str, name: &str) {
        self.code_to_name.insert(code.to_string(), name.to_string());
        self.name_to_code.insert(name.to_string(), code.to_string());
        if let Err(e) = self.save() {
            tracing::warn!(error = %e, "Failed to save cache");
        }
    }

    fn save(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.cache_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.code_to_name)?;
        fs::write(&self.cache_path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_operations() {
        let mut cache = FundCache::new();
        cache.set_mapping("000001", "测试基金");
        assert_eq!(cache.get_name("000001"), Some("测试基金".to_string()));
        assert_eq!(cache.get_code("测试基金"), Some("000001".to_string()));
    }
}
