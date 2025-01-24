use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::info;

const DEFAULT_API_URL: &str = "https://cursor-account-api.vercel.app/account/random";

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiConfig {
    pub url: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            url: DEFAULT_API_URL.to_string(),
        }
    }
}

pub struct ApiConfigManager {
    config_path: PathBuf,
}

impl ApiConfigManager {
    pub fn new() -> Result<Self> {
        let config_path = if cfg!(windows) {
            dirs::config_dir()
                .expect("Failed to get config directory")
                .join("Cursor")
                .join("api_config.json")
        } else if cfg!(target_os = "macos") {
            dirs::home_dir()
                .expect("Failed to get home directory")
                .join("Library")
                .join("Application Support")
                .join("Cursor")
                .join("api_config.json")
        } else {
            dirs::config_dir()
                .expect("Failed to get config directory")
                .join("cursor")
                .join("api_config.json")
        };

        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        Ok(Self { config_path })
    }

    pub fn load(&self) -> Result<ApiConfig> {
        if !self.config_path.exists() {
            info!("Config file not found, using default configuration");
            return Ok(ApiConfig::default());
        }

        let content = fs::read_to_string(&self.config_path)?;
        let config: ApiConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, config: &ApiConfig) -> Result<()> {
        let content = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, content)?;
        info!("API configuration saved successfully");
        Ok(())
    }

    pub fn reset(&self) -> Result<ApiConfig> {
        let config = ApiConfig::default();
        self.save(&config)?;
        info!("API configuration reset to default");
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_api_config_manager() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("api_config.json");

        let manager = ApiConfigManager {
            config_path: config_path.clone(),
        };

        // Test default config
        let config = manager.load().unwrap();
        assert_eq!(config.url, DEFAULT_API_URL);

        // Test saving config
        let new_config = ApiConfig {
            url: "https://example.com/api".to_string(),
        };
        manager.save(&new_config).unwrap();

        // Test loading saved config
        let loaded_config = manager.load().unwrap();
        assert_eq!(loaded_config.url, new_config.url);

        // Test resetting config
        let reset_config = manager.reset().unwrap();
        assert_eq!(reset_config.url, DEFAULT_API_URL);
    }
}
