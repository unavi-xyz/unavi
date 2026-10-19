//! Build-time configuration, from `secretspec.toml` under the profile
//! `SECRETSPEC_PROFILE` names when the app is compiled.

use bevy::log::error;

use crate::identity::SyncConfig;

unavi_config::config!("secretspec.toml");

/// Whether this binary is a development build. Diagnostics and lenient
/// runtime behaviour key off this rather than the cargo profile, which a
/// release-inheriting `web-dev` build would otherwise hide.
#[must_use]
pub fn dev_build() -> bool {
    Config::load().is_ok_and(|config| config.unavi_dev == "true")
}

/// The registries to follow. A config that fails to load follows none, and
/// the app runs peer to peer.
pub fn sync_config() -> SyncConfig {
    let config = match Config::load() {
        Ok(config) => config,
        Err(err) => {
            error!(%err, "no registry to follow");
            return SyncConfig::default();
        }
    };

    SyncConfig {
        targets:          config
            .unavi_sync_targets
            .split(',')
            .map(str::trim)
            .filter(|did| !did.is_empty())
            .map(str::to_owned)
            .collect(),
        allow_local_dids: config.unavi_allow_local_dids == "true",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_declared_targets_name_servers() {
        let config = sync_config();

        assert!(
            !config.targets.is_empty(),
            "no sync target declared; the manifest may have renamed one"
        );
        assert!(config.targets.iter().all(|did| did.starts_with("did:")));
    }
}
