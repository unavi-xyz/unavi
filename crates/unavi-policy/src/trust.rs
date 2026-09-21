use std::{
    collections::HashMap,
    sync::Arc,
};

use anyhow::Context;
use bevy::prelude::*;
use iroh::EndpointId;
use parking_lot::RwLock;
use ron::Options;
use serde::{
    Deserialize,
    Serialize,
};
use unavi_identity::auth::bindings::Bindings;
use unavi_local::LocalStorage;
use xdid::core::did::Did;

/// How far a peer is trusted.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Trust {
    Blocked,
    #[default]
    Guest,
    Trusted,
    Myself,
}

impl Trust {
    /// Whether this level meets a requirement of `required`. `Blocked` meets
    /// nothing, including `Blocked`.
    #[must_use]
    pub const fn clears(self, required: Self) -> bool {
        !matches!(self, Self::Blocked) && (self as u8) >= (required as u8)
    }
}

/// The trust a document requires of whoever writes it.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Threshold(pub Trust);

const TABLE_KEY: &str = "trust.ron";

/// Per-peer trust levels, keyed by DID.
#[derive(Resource, Clone)]
pub struct TrustTable(Arc<Inner>);

struct Inner {
    overrides: RwLock<HashMap<Did, Trust>>,
    storage:   LocalStorage,
}

#[derive(Default, Serialize, Deserialize)]
struct Stored {
    #[serde(default)]
    peers: HashMap<String, Trust>,
}

impl TrustTable {
    #[must_use]
    pub fn new(storage: LocalStorage) -> Self {
        Self(Arc::new(Inner {
            overrides: RwLock::default(),
            storage,
        }))
    }

    /// Reads the table from `storage`. Entries that do not parse as DIDs are
    /// dropped. A file that does not parse is an error.
    pub fn load(storage: LocalStorage) -> anyhow::Result<Self> {
        let mut overrides = HashMap::new();

        if let Some(text) = storage.read(TABLE_KEY)? {
            let stored: Stored = Options::default()
                .from_str(&text)
                .with_context(|| format!("parse {TABLE_KEY}"))?;
            for (did, trust) in stored.peers {
                match did.parse::<Did>() {
                    Ok(did) => {
                        overrides.insert(did, trust);
                    }
                    Err(err) => tracing::warn!(?err, %did, "dropping unparseable trust entry"),
                }
            }
        }

        Ok(Self(Arc::new(Inner {
            overrides: RwLock::new(overrides),
            storage,
        })))
    }

    #[must_use]
    pub fn of_peer(&self, peer: EndpointId, bindings: &Bindings) -> Trust {
        bindings
            .did_of(peer)
            .map_or_default(|did| self.of_did(&did))
    }

    #[must_use]
    pub fn of_did(&self, did: &Did) -> Trust {
        self.0
            .overrides
            .read()
            .get(did)
            .copied()
            .unwrap_or_default()
    }

    pub fn set(&self, did: Did, trust: Trust) {
        self.0.overrides.write().insert(did, trust);
    }

    pub fn clear(&self, did: &Did) {
        self.0.overrides.write().remove(did);
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let stored = Stored {
            peers: self
                .0
                .overrides
                .read()
                .iter()
                .map(|(did, trust)| (did.to_string(), *trust))
                .collect(),
        };

        let text =
            Options::default().to_string_pretty(&stored, ron::ser::PrettyConfig::default())?;
        self.0.storage.write(TABLE_KEY, &text)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        str::FromStr,
    };

    use iroh::SecretKey;

    use super::*;

    fn peer() -> EndpointId {
        SecretKey::generate().public()
    }

    /// A fresh table on disk, distinct per test so parallel runs never share a
    /// file.
    fn storage() -> (PathBuf, LocalStorage) {
        let dir = std::env::temp_dir().join(format!(
            "unavi-trust-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("unnamed")
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let storage = LocalStorage::Path(dir.clone());
        (dir, storage)
    }

    #[test]
    fn an_unproven_peer_is_a_guest() {
        let table = TrustTable::new(LocalStorage::default());
        assert_eq!(table.of_peer(peer(), &Bindings::default()), Trust::Guest);
    }

    #[test]
    fn a_trust_level_survives_the_endpoint_it_was_learned_on() {
        let table = TrustTable::new(LocalStorage::default());
        let did = Did::from_str("did:web:example.com").expect("did");
        let bindings = Bindings::default();
        table.set(did.clone(), Trust::Trusted);

        let (first, second) = (peer(), peer());
        bindings.bind(first, did.clone());
        assert_eq!(table.of_peer(first, &bindings), Trust::Trusted);

        bindings.unbind(first);
        bindings.bind(second, did);
        assert_eq!(
            table.of_peer(second, &bindings),
            Trust::Trusted,
            "the same DID on a new endpoint keeps its trust level"
        );
    }

    #[test]
    fn the_table_survives_a_round_trip_through_disk() {
        let (dir, storage) = storage();

        let blocked = Did::from_str("did:web:blocked.example").expect("did");
        let table = TrustTable::new(storage.clone());
        table.set(blocked.clone(), Trust::Blocked);
        table.save().expect("save");

        let table = TrustTable::load(storage).expect("load");

        assert_eq!(table.of_did(&blocked), Trust::Blocked);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_corrupt_table_is_an_error_not_an_empty_start() {
        let (dir, storage) = storage();

        assert!(
            TrustTable::load(storage.clone()).is_ok(),
            "a first run has no table and that is not a failure"
        );

        std::fs::write(dir.join(TABLE_KEY), "(peers: [[[").expect("write");
        assert!(
            TrustTable::load(storage).is_err(),
            "coming up clean would silently unblock every ejected peer"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn the_ladder_is_ordered_from_blocked_up() {
        assert!(Trust::Blocked < Trust::Guest);
        assert!(Trust::Guest < Trust::Trusted);
        assert!(Trust::Trusted < Trust::Myself);
    }

    #[test]
    fn a_blocked_peer_clears_nothing() {
        for required in [Trust::Blocked, Trust::Guest, Trust::Trusted, Trust::Myself] {
            assert!(
                !Trust::Blocked.clears(required),
                "blocked must not clear {required:?}"
            );
        }
    }

    #[test]
    fn the_default_trust_level_clears_the_open_default() {
        assert!(Trust::default().clears(Threshold::default().0));
        assert!(!Trust::Guest.clears(Trust::Trusted));
        assert!(Trust::Myself.clears(Trust::Trusted));
    }

    #[test]
    fn an_own_only_threshold_refuses_everyone_below_the_local_user() {
        let own_only = Threshold(Trust::Myself);
        assert!(!Trust::Guest.clears(own_only.0));
        assert!(!Trust::Trusted.clears(own_only.0));
        assert!(Trust::Myself.clears(own_only.0));
    }
}
