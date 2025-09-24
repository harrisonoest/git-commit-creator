use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub commit_prefixes: Vec<String>,
    pub branch_prefixes: Vec<String>,
    pub story_prefix: Option<String>,
    pub auto_push: Option<bool>,
    pub default_commit_prefix: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            commit_prefixes: vec![
                "feat:".to_string(),
                "fix:".to_string(),
                "docs:".to_string(),
                "style:".to_string(),
                "refactor:".to_string(),
                "test:".to_string(),
                "ci:".to_string(),
                "chore:".to_string(),
            ],
            branch_prefixes: vec![
                "build".to_string(),
                "chore".to_string(),
                "ci".to_string(),
                "docs".to_string(),
                "feat".to_string(),
                "fix".to_string(),
                "perf".to_string(),
                "refactor".to_string(),
                "revert".to_string(),
                "style".to_string(),
                "test".to_string(),
            ],
            story_prefix: None,
            auto_push: Some(true),
            default_commit_prefix: None,
        }
    }
}

impl Config {
    /// Loads config from ~/.gitcc/config.toml or creates default if not found
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Saves config to ~/.gitcc/config.toml
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    /// Returns path to config file
    fn config_path() -> Result<PathBuf> {
        let home =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".gitcc").join("config.toml"))
    }
}
