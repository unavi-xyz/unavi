use bevy::prelude::*;
use ron::{
    Options,
    extensions::Extensions,
    ser::PrettyConfig,
};
use unavi_store::local::Storage;

use crate::config::{
    InputConfig,
    patch::ConfigPatch,
};

const KEY: &str = "input.ron";

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

/// Reads the config, writing the defaults out first if there is nothing
/// there. A value that fails to parse or to read is left alone — overwriting
/// it would throw away the edit that broke it.
#[must_use]
pub fn load(storage: &Storage) -> InputConfig {
    match storage.read(KEY) {
        Ok(Some(text)) => match parse(&text) {
            Ok(config) => config,
            Err(err) => {
                error!(
                    ?err,
                    "input config failed to parse, falling back to defaults"
                );
                InputConfig::default()
            }
        },
        Ok(None) => {
            let config = InputConfig::default();
            save(storage, &config);
            config
        }
        Err(err) => {
            error!(?err, "input config is unreadable, falling back to defaults");
            InputConfig::default()
        }
    }
}

pub fn save(storage: &Storage, config: &InputConfig) {
    let text = match to_text(config) {
        Ok(text) => text,
        Err(err) => {
            error!(%err, "failed to serialize input config");
            return;
        }
    };

    if let Err(err) = storage.write(KEY, &text) {
        error!(?err, "failed to write input config");
    }
}
