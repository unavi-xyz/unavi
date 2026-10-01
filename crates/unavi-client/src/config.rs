//! Build-time configuration, from `secretspec.toml` under the profile
//! `SECRETSPEC_PROFILE` names when the client is compiled.

use bevy::log::error;

use crate::identity::SyncConfig;

unavi_config::config!("secretspec.toml");

/// The registries to follow. A config that fails to load follows none, and
/// the client runs peer to peer.
pub fn sync_config() -> SyncConfig {
    let config = match Config::load() {
        Ok(config) => config,
        Err(err) => {
            error!(%err, "no registry to follow");
            return SyncConfig::default();
        }
    };

    SyncConfig {
        targets: config
            .unavi_sync_targets
            .split(',')
            .map(str::trim)
            .filter(|did| !did.is_empty())
            .map(str::to_owned)
            .collect(),
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
