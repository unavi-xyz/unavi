use std::{fs, sync::Arc};

use anyhow::Context;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::DIRS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateChannel {
    Stable,
    Beta,
}

impl UpdateChannel {
    #[must_use]
    pub const fn is_beta(self) -> bool {
        matches!(self, Self::Beta)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub update_channel: UpdateChannel,
    #[serde(default)]
    pub xr_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            update_channel: UpdateChannel::Beta,
            xr_mode: false,
        }
    }
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

    /// Save the configuration to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails or the file cannot be written.
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

    /// Update the configuration using the provided function and save to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if saving the configuration fails.
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
