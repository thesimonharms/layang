use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const PLACEHOLDER_EDITOR: &str = "<PATH TO EDITOR HERE>";
const PLACEHOLDER_API_URL: &str = "<API URL HERE>";

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub editor: String,
    pub api_url: String,
}

pub fn config_path() -> Result<PathBuf> {
    let dir = dirs::config_dir().ok_or_else(|| anyhow!("Could not determine config directory"))?;
    Ok(dir.join("layang").join("config.toml"))
}

pub fn load_or_create_config() -> Result<Config> {
    let path = config_path()?;

    if !path.exists() {
        // Create directory and file with placeholders
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }

        let default_config = Config {
            editor: PLACEHOLDER_EDITOR.to_string(),
            api_url: PLACEHOLDER_API_URL.to_string(),
        };

        let toml_str = toml::to_string(&default_config)
            .context("Failed to serialize default config")?;

        std::fs::write(&path, toml_str)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        println!("Config file created at: {}", path.display());
        println!("Please edit it and set your editor path and API URL, then run layang again.");
        std::process::exit(0);
    }

    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let config: Config = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

    if config.editor == PLACEHOLDER_EDITOR {
        return Err(anyhow!(
            "Config error: 'editor' is still the placeholder value.\nPlease edit: {}",
            path.display()
        ));
    }

    if config.api_url == PLACEHOLDER_API_URL {
        return Err(anyhow!(
            "Config error: 'api_url' is still the placeholder value.\nPlease edit: {}",
            path.display()
        ));
    }

    if !std::path::Path::new(&config.editor).exists() {
        return Err(anyhow!(
            "Config error: editor '{}' does not exist on disk.\nPlease edit: {}",
            config.editor,
            path.display()
        ));
    }

    Ok(config)
}
