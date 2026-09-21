use std::{
    collections::HashMap,
    sync::Arc,
};

use anyhow::Context;
use bevy::prelude::Resource;
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

/// Local opinion of a peer's trust. Used to restrict capabilities.
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
    /// Whether a peer at this trust level clears a capability needing
    /// `required`.
    ///
    /// [`Trust::Blocked`] clears nothing, including a requirement of
    /// `Blocked`, so a floor of `Guest` cannot be undercut by naming the
    /// lowest trust level.
    #[must_use]
    pub const fn clears(self, required: Self) -> bool {
        !matches!(self, Self::Blocked) && (self as u8) >= (required as u8)
    }
}

#[derive(Resource, Clone)]
pub struct TrustTable(Arc<TrustTableInner>);

struct TrustTableInner {
    overrides: RwLock<HashMap<Did, Trust>>,
    storage:   LocalStorage,
}

const TABLE_KEY: &str = "trust.ron";

#[derive(Default, Serialize, Deserialize)]
struct Stored {
    #[serde(default)]
    peers: HashMap<String, Trust>,
}

impl TrustTable {
    #[must_use]
    pub fn new(storage: LocalStorage) -> Self {
        Self(Arc::new(TrustTableInner {
            overrides: RwLock::default(),
            storage,
        }))
    }

    /// Loads the table from `storage`, discarding entries that no
    /// longer parse as DIDs rather than refusing the whole file.
    pub fn load(storage: LocalStorage) -> anyhow::Result<Self> {
        let stored = match read_table(&storage, TABLE_KEY) {
            Ok(None) => Stored::default(),
            Ok(Some(stored)) => stored,
            Err(err) => return Err(err),
        };

        let mut overrides = HashMap::new();
        for (did, trust) in stored.peers {
            match did.parse::<Did>() {
                Ok(did) => {
                    overrides.insert(did, trust);
                }
                Err(err) => tracing::warn!(?err, %did, "dropping unparseable trust entry"),
            }
        }

        Ok(Self(Arc::new(TrustTableInner {
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

    /// Writes the table to local storage.
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

/// `Ok(None)` when nothing is recorded at `key`.
///
/// A table that is present but will not parse is an `Err`. A truncated write
/// leaves valid UTF-8 that is not valid RON.
fn read_table(storage: &LocalStorage, key: &str) -> anyhow::Result<Option<Stored>> {
    let Some(text) = storage.read(key)? else {
        return Ok(None);
    };
    Ok(Some(
        Options::default()
            .from_str(&text)
            .with_context(|| format!("parse {key}"))?,
    ))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use iroh::{
        EndpointId,
        SecretKey,
    };

    use super::*;

    fn peer() -> EndpointId {
        SecretKey::generate().public()
    }

    /// A fresh table on disk, distinct per test so parallel runs never share a
    /// file.
    fn storage() -> (std::path::PathBuf, LocalStorage) {
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

        std::fs::write(dir.join("trust.ron"), "(peers: [[[").expect("write");
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
        assert!(Trust::default().clears(Trust::Guest));
        assert!(!Trust::Guest.clears(Trust::Trusted));
        assert!(Trust::Myself.clears(Trust::Trusted));
    }
}
