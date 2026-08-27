use crate::identity::SyncConfig;

unavi_secrets::declare!("secretspec.toml");

pub fn sync_config() -> SyncConfig {
    let secrets = Secrets::load();

    SyncConfig {
        targets: secrets
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
