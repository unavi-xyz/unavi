//! Server configuration: the build-time `secretspec.toml` values, and the
//! operator's registry policy read from the data directory.

use std::{
    path::Path,
    str::FromStr,
};

use anyhow::Context;
use iroh_docs::NamespaceId;
use serde::Deserialize;
use unavi_registry::server::{
    Config as RegistryConfig,
    Submitters,
};
use xdid::core::did::Did;

unavi_config::config!("secretspec.toml");

/// Where the operator's registry policy lives, under the data directory.
pub const REGISTRY_FILE: &str = "registry.ron";

/// The hand-edited form of [`RegistryConfig`]. Every field is optional; an
/// absent file is the defaults.
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RegistryFile {
    /// DIDs allowed to submit. Absent means any authenticated DID.
    allowlist:  Option<Vec<String>>,
    /// Namespace ids to feature.
    featured:   Vec<String>,
    /// Tags recognized as categories.
    categories: Vec<String>,
}

/// Reads [`REGISTRY_FILE`] from `data_dir`, or the defaults when there is no
/// data directory or no file.
pub fn registry_config(data_dir: Option<&Path>) -> anyhow::Result<RegistryConfig> {
    let mut config = RegistryConfig::default();

    let Some(path) = data_dir.map(|dir| dir.join(REGISTRY_FILE)) else {
        return Ok(config);
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(config),
        Err(err) => return Err(err).with_context(|| format!("read {}", path.display())),
    };
    let file: RegistryFile =
        ron::from_str(&text).with_context(|| format!("parse {}", path.display()))?;

    if let Some(allowlist) = file.allowlist {
        config.submitters = Submitters::Allowlist(
            allowlist
                .iter()
                .map(|did| Did::from_str(did).with_context(|| format!("allowlist DID {did}")))
                .collect::<anyhow::Result<_>>()?,
        );
    }
    config.featured = file
        .featured
        .iter()
        .map(|ns| NamespaceId::from_str(ns).with_context(|| format!("featured namespace {ns}")))
        .collect::<anyhow::Result<_>>()?;
    config.categories = file.categories.into_iter().map(Into::into).collect();

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_absent_file_is_the_defaults() {
        let dir = tempfile::tempdir().expect("temp dir");
        let config = registry_config(Some(dir.path())).expect("defaults");
        assert!(matches!(config.submitters, Submitters::Open));
    }

    #[test]
    fn the_file_sets_operator_policy() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(
            dir.path().join(REGISTRY_FILE),
            r#"(allowlist: Some(["did:web:example.com"]), categories: ["social"])"#,
        )
        .expect("write");

        let config = registry_config(Some(dir.path())).expect("parse");
        assert!(matches!(config.submitters, Submitters::Allowlist(ref dids) if dids.len() == 1));
        assert_eq!(config.categories, ["social"]);
    }
}
