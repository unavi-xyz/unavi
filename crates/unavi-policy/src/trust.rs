use std::{
    collections::HashMap,
    sync::Arc,
};

use anyhow::Context;
use bevy::prelude::Resource;
use iroh::EndpointId;
use parking_lot::RwLock;
use serde::{
    Deserialize,
    Serialize,
};
use unavi_identity::auth::bindings::Bindings;
use unavi_store::local::Storage;
use xdid::core::did::Did;

/// How much a peer is trusted, as one ordinal rung.
///
/// The opinion is the local viewer's and is never gossiped as authoritative.
/// Ranks *peers*, not documents — [`crate::tier::Tier`] is the document side.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Trust {
    /// Ejected. Below the floor every capability sits at, so naming it as a
    /// minimum anywhere would be a mistake.
    Blocked,
    /// Anyone else present. The default, and the rung a normal item must work
    /// at with no configuration and no prompt.
    #[default]
    Guest,
    /// Marked by the local user.
    Trusted,
    /// The local user.
    Myself,
}

impl Trust {
    /// Whether a peer at this rung clears a capability needing `required`.
    ///
    /// [`Trust::Blocked`] clears nothing, including a requirement of
    /// `Blocked`, so a floor of `Guest` cannot be undercut by naming the
    /// bottom rung.
    #[must_use]
    pub const fn clears(self, required: Self) -> bool {
        !matches!(self, Self::Blocked) && (self as u8) >= (required as u8)
    }
}

/// The rungs the local user set by hand, keyed by DID, and the storage they
/// persist through.
///
/// Keyed to the DID rather than the endpoint because an `EndpointId` rotates,
/// and a table keyed to one would forget every peer on their next device.
#[derive(Resource, Clone)]
pub struct TrustTable(Arc<Inner>);

struct Inner {
    overrides: RwLock<HashMap<Did, Trust>>,
    storage:   Storage,
}

/// The table's key in a [`Storage`], and the previous good copy kept beside it.
const TABLE_KEY: &str = "trust.toml";
const BACKUP_KEY: &str = "trust.toml.bak";

#[derive(Default, Serialize, Deserialize)]
struct Stored {
    #[serde(default)]
    peers: HashMap<String, Trust>,
}

impl TrustTable {
    #[must_use]
    pub fn new(storage: Storage) -> Self {
        Self(Arc::new(Inner {
            overrides: RwLock::default(),
            storage,
        }))
    }

    /// Loads the manual rungs from `storage`, discarding entries that no
    /// longer parse as DIDs rather than refusing the whole file.
    ///
    /// A table that cannot be read at all is an error rather than an empty
    /// start: coming up clean would silently un-block every peer the user
    /// ejected. The previous good copy is tried first so a truncated write is
    /// survivable.
    pub fn load(storage: Storage) -> anyhow::Result<Self> {
        let stored = match read_table(&storage, TABLE_KEY) {
            Ok(None) => Stored::default(),
            Ok(Some(stored)) => stored,
            Err(err) => {
                tracing::warn!(?err, "trust table unreadable, falling back to the backup");
                read_table(&storage, BACKUP_KEY)?
                    .ok_or_else(|| err.context("no backup trust table exists"))?
            }
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

        Ok(Self(Arc::new(Inner {
            overrides: RwLock::new(overrides),
            storage,
        })))
    }

    /// The rung `peer` sits at.
    ///
    /// A peer that has proved no DID cannot rise above [`Trust::Guest`]: an
    /// unproven claim is not an identity, so there is nothing to have an
    /// opinion about.
    #[must_use]
    pub fn of_peer(&self, peer: EndpointId, bindings: &Bindings) -> Trust {
        bindings
            .did_of(peer)
            .map_or(Trust::Guest, |did| self.of_did(&did))
    }

    /// What the user said about `did`, or [`Trust::Guest`] if they have said
    /// nothing.
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

    /// Writes the previous good copy aside, then replaces the table through
    /// the backend's atomic write. A trust table half-written by a crash
    /// would read as no blocks at all.
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

        let text = toml::to_string_pretty(&stored)?;
        // Whatever is current now becomes the fallback; an unreadable
        // current is replaced rather than carried forward.
        if let Ok(Some(previous)) = self.0.storage.read(TABLE_KEY) {
            self.0.storage.write(BACKUP_KEY, &previous)?;
        }

        self.0.storage.write(TABLE_KEY, &text)
    }
}

/// `Ok(None)` when nothing is recorded at `key`.
///
/// A table that is present but will not parse is an `Err`, the same answer as
/// one that cannot be read at all. A truncated write leaves valid UTF-8 that is
/// not valid TOML, so treating the two alike is what makes the backup reachable
/// in the case it exists for.
fn read_table(storage: &Storage, key: &str) -> anyhow::Result<Option<Stored>> {
    let Some(text) = storage.read(key)? else {
        return Ok(None);
    };
    Ok(Some(
        toml::from_str(&text).with_context(|| format!("parse {key}"))?,
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
    fn storage() -> (std::path::PathBuf, Storage) {
        let dir = std::env::temp_dir().join(format!(
            "unavi-trust-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("unnamed")
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let storage = Storage::Path(dir.clone());
        (dir, storage)
    }

    #[test]
    fn an_unproven_peer_is_a_guest() {
        let table = TrustTable::new(Storage::Ephemeral);
        assert_eq!(table.of_peer(peer(), &Bindings::default()), Trust::Guest);
    }

    #[test]
    fn a_rung_survives_the_endpoint_it_was_learned_on() {
        let table = TrustTable::new(Storage::Ephemeral);
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
            "the same DID on a new endpoint keeps its rung"
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

        std::fs::write(dir.join("trust.toml"), "peers = [[[").expect("write");
        assert!(
            TrustTable::load(storage).is_err(),
            "coming up clean would silently unblock every ejected peer"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_truncated_write_leaves_the_previous_table_readable() {
        let (dir, storage) = storage();

        let blocked = Did::from_str("did:web:kept.example").expect("did");
        let table = TrustTable::new(storage.clone());
        table.set(blocked.clone(), Trust::Blocked);
        table.save().expect("save");
        table
            .save()
            .expect("save again, rotating the table into the backup");

        std::fs::write(dir.join("trust.toml"), "peers = [[[").expect("truncate");
        let table = TrustTable::load(storage).expect("the backup carries the table");

        assert_eq!(table.of_did(&blocked), Trust::Blocked);

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
    fn the_default_rung_clears_the_open_default() {
        assert!(Trust::default().clears(Trust::Guest));
        assert!(!Trust::Guest.clears(Trust::Trusted));
        assert!(Trust::Myself.clears(Trust::Trusted));
    }
}
