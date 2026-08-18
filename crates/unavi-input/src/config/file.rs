#[cfg(not(target_family = "wasm"))]
use std::{
    fs,
    path::PathBuf,
};

use bevy::prelude::*;
use ron::{
    Options,
    extensions::Extensions,
    ser::PrettyConfig,
};

use crate::config::{
    InputConfig,
    patch::ConfigPatch,
};

#[cfg(not(target_family = "wasm"))]
const FILE_NAME: &str = "input.ron";

/// `implicit_some` so an optional binding reads as the list it is rather than
/// as `Some([...])`.
fn options() -> Options {
    Options::default().with_default_extension(Extensions::IMPLICIT_SOME)
}

pub fn parse(text: &str) -> Result<InputConfig, ron::error::SpannedError> {
    options()
        .from_str::<ConfigPatch>(text)
        .map(ConfigPatch::resolve)
}

pub fn to_text(config: &InputConfig) -> Result<String, ron::Error> {
    options().to_string_pretty(
        &ConfigPatch::from(config),
        PrettyConfig::default().extensions(Extensions::IMPLICIT_SOME),
    )
}

#[cfg(not(target_family = "wasm"))]
#[must_use]
pub fn path() -> PathBuf {
    unavi_util::dirs::config_dir().join(FILE_NAME)
}

/// Reads the config, writing the defaults out first if there is nothing there.
///
/// A file that fails to parse is left alone — overwriting it would throw away
/// the edit that broke it, which is the one thing the author wants back.
#[cfg(not(target_family = "wasm"))]
#[must_use]
pub fn load() -> InputConfig {
    let path = path();

    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => {
            info!(?path, %err, "writing default input config");
            let config = InputConfig::default();
            save(&config);
            return config;
        }
    };

    match parse(&text) {
        Ok(config) => config,
        Err(err) => {
            error!(?path, %err, "input config failed to parse, falling back to defaults");
            InputConfig::default()
        }
    }
}

#[cfg(not(target_family = "wasm"))]
pub fn save(config: &InputConfig) {
    let path = path();

    let text = match to_text(config) {
        Ok(text) => text,
        Err(err) => {
            error!(%err, "failed to serialize input config");
            return;
        }
    };

    if let Err(err) = fs::write(&path, text) {
        error!(?path, %err, "failed to write input config");
    }
}
