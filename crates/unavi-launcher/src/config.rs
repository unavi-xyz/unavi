use std::{
    fs,
    sync::Arc,
};

use anyhow::Context;
use parking_lot::Mutex;
use serde::{
    Deserialize,
    Serialize,
};
use tracing::info;

use crate::DIRS;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub xr_mode: bool,
}

impl Config {
    fn config_path() -> std::path::PathBuf {
        DIRS.data_local_dir().join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => config,
                Err(e) => {
                    tracing::warn!("failed to parse config file: {e}, using defaults");
                    Self::default()
                }
            },
            Err(e) => {
                tracing::warn!("failed to read config file: {e}, using defaults");
                Self::default()
            }
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        let contents = toml::to_string_pretty(self).context("serialize config")?;
        info!("writing config to {}", path.to_string_lossy());
        fs::write(&path, contents).context("write config file")?;
        Ok(())
    }
}

pub struct ConfigStore {
    config: Arc<Mutex<Config>>,
}

impl ConfigStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(Config::load())),
        }
    }

    /// Get a copy of the current configuration.
    #[must_use]
    pub fn get(&self) -> Config {
        self.config.lock().clone()
    }

    /// Updates the configuration via `f`, saving it to disk.
    pub fn update<F>(&self, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut Config),
    {
        let mut config = self.config.lock();
        f(&mut config);
        config.save()?;
        drop(config);
        Ok(())
    }
}

impl Default for ConfigStore {
    fn default() -> Self {
        Self::new()
    }
}
