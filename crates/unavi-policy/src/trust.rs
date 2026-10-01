//! How far each peer is trusted, keyed by the DID it proved.

use std::{
    collections::{
        HashMap,
        HashSet,
    },
    sync::Arc,
};

use anyhow::Context;
use bevy::ecs::resource::Resource;
use iroh::EndpointId;
use parking_lot::RwLock;
use ron::Options;
use serde::{
    Deserialize,
    Serialize,
};
use unavi_identity::auth::Bindings;
use unavi_local::DeviceStorage;
use xdid::core::did::Did;

/// How far a peer is trusted, lowest first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Trust {
    Blocked,
    /// A peer that proved no DID, or one judged before the local identity is
    /// ready. Below every DID, so proving one never costs a peer anything.
    Anonymous,
    /// A DID nobody has judged, unless [`TrustTable::default_trust`] says
    /// otherwise.
    Guest,
    Trusted,
    /// The local user. Decided by endpoint, never assigned.
    Myself,
}

/// [`Trust::Myself`] belongs to this endpoint alone, so a table refuses to
/// grant it to a DID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("Myself is decided by endpoint and cannot be assigned")]
pub struct UnassignableTrust;

const TABLE_KEY: &str = "trust.ron";

/// Per-peer trust levels, keyed by DID.
///
/// A DID costs nothing to mint, so blocking one stops that identity, not its
/// owner. Setting [`Self::set_default_trust`] to [`Trust::Anonymous`] makes
/// every unjudged DID count as unproven, which is what gives a block teeth.
/// [`Self::block_endpoint`] covers a peer with no DID, for the session.
#[derive(Resource, Clone)]
pub struct TrustTable(Arc<Inner>);

struct Inner {
    overrides:       RwLock<HashMap<Did, Trust>>,
    default_trust:   RwLock<Trust>,
    blocked_devices: RwLock<HashSet<EndpointId>>,
    storage:         DeviceStorage,
}

#[derive(Serialize, Deserialize)]
struct Stored {
    #[serde(default)]
    peers:         HashMap<String, Trust>,
    #[serde(default = "default_trust")]
    default_trust: Trust,
}

const fn default_trust() -> Trust {
    Trust::Guest
}

impl TrustTable {
    #[must_use]
    pub fn new(storage: DeviceStorage) -> Self {
        Self::with(HashMap::new(), default_trust(), storage)
    }

    fn with(overrides: HashMap<Did, Trust>, default: Trust, storage: DeviceStorage) -> Self {
        Self(Arc::new(Inner {
            overrides: RwLock::new(overrides),
            default_trust: RwLock::new(default),
            blocked_devices: RwLock::default(),
            storage,
        }))
    }

    /// Reads the table from `storage`. Entries that do not parse as DIDs, or
    /// that claim [`Trust::Myself`], are dropped. A file that does not parse
    /// is an error.
    pub fn load(storage: DeviceStorage) -> anyhow::Result<Self> {
        let Some(text) = storage.read(TABLE_KEY)? else {
            return Ok(Self::new(storage));
        };

        let stored: Stored = Options::default()
            .from_str(&text)
            .with_context(|| format!("parse {TABLE_KEY}"))?;

        let mut overrides = HashMap::new();
        for (did, trust) in stored.peers {
            if trust == Trust::Myself {
                tracing::warn!(%did, "dropping a trust entry that claims Myself");
                continue;
            }
            match did.parse::<Did>() {
                Ok(did) => {
                    overrides.insert(did, trust);
                }
                Err(err) => tracing::warn!(?err, %did, "dropping unparseable trust entry"),
            }
        }

        let default = if stored.default_trust == Trust::Myself {
            default_trust()
        } else {
            stored.default_trust
        };

        Ok(Self::with(overrides, default, storage))
    }

    /// How far `peer` is trusted: blocked if its endpoint is, its DID's level
    /// if it proved one, and [`Trust::Anonymous`] otherwise.
    #[must_use]
    pub fn of_peer(&self, peer: EndpointId, bindings: &Bindings) -> Trust {
        if self.is_endpoint_blocked(peer) {
            return Trust::Blocked;
        }
        bindings
            .did_of(peer)
            .map_or(Trust::Anonymous, |did| self.of_did(&did))
    }

    #[must_use]
    pub fn of_did(&self, did: &Did) -> Trust {
        self.0
            .overrides
            .read()
            .get(did)
            .copied()
            .unwrap_or_else(|| self.default_trust())
    }

    pub fn set(&self, did: Did, trust: Trust) -> Result<(), UnassignableTrust> {
        if trust == Trust::Myself {
            return Err(UnassignableTrust);
        }
        self.0.overrides.write().insert(did, trust);
        Ok(())
    }

    pub fn clear(&self, did: &Did) {
        self.0.overrides.write().remove(did);
    }

    /// The level of a DID nobody has judged.
    #[must_use]
    pub fn default_trust(&self) -> Trust {
        *self.0.default_trust.read()
    }

    pub fn set_default_trust(&self, trust: Trust) -> Result<(), UnassignableTrust> {
        if trust == Trust::Myself {
            return Err(UnassignableTrust);
        }
        *self.0.default_trust.write() = trust;
        Ok(())
    }

    /// Blocks `peer`'s endpoint until the process exits, whatever DID it
    /// proves. For a peer with no DID to block.
    pub fn block_endpoint(&self, peer: EndpointId) {
        self.0.blocked_devices.write().insert(peer);
    }

    pub fn unblock_endpoint(&self, peer: EndpointId) {
        self.0.blocked_devices.write().remove(&peer);
    }

    #[must_use]
    pub fn is_endpoint_blocked(&self, peer: EndpointId) -> bool {
        self.0.blocked_devices.read().contains(&peer)
    }

    /// Persists the DID levels and the default. Endpoint blocks are not saved.
    pub fn save(&self) -> anyhow::Result<()> {
        let stored = Stored {
            peers:         self
                .0
                .overrides
                .read()
                .iter()
                .map(|(did, trust)| (did.to_string(), *trust))
                .collect(),
            default_trust: self.default_trust(),
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
    fn storage() -> (PathBuf, DeviceStorage) {
        let dir = std::env::temp_dir().join(format!(
            "unavi-trust-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("unnamed")
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let storage = DeviceStorage::at(dir.clone());
        (dir, storage)
    }

    #[test]
    fn an_unproven_peer_is_anonymous() {
        let table = TrustTable::new(DeviceStorage::memory());
        assert_eq!(
            table.of_peer(peer(), &Bindings::default()),
            Trust::Anonymous
        );
    }

    #[test]
    fn an_unjudged_did_takes_the_default() {
        let table = TrustTable::new(DeviceStorage::memory());
        let did = Did::from_str("did:web:example.com").expect("did");
        assert_eq!(table.of_did(&did), Trust::Guest);

        table
            .set_default_trust(Trust::Anonymous)
            .expect("assignable");
        assert_eq!(
            table.of_did(&did),
            Trust::Anonymous,
            "a known-DIDs-only table treats a fresh DID as unproven"
        );
    }

    #[test]
    fn myself_cannot_be_assigned() {
        let table = TrustTable::new(DeviceStorage::memory());
        let did = Did::from_str("did:web:example.com").expect("did");
        assert_eq!(table.set(did, Trust::Myself), Err(UnassignableTrust));
    }

    #[test]
    fn an_endpoint_block_holds_whatever_did_is_proved() {
        let table = TrustTable::new(DeviceStorage::memory());
        let bindings = Bindings::default();
        let (device, did) = (peer(), Did::from_str("did:web:example.com").expect("did"));
        table.set(did.clone(), Trust::Trusted).expect("assignable");
        bindings.bind(device, did);

        table.block_endpoint(device);

        assert_eq!(table.of_peer(device, &bindings), Trust::Blocked);
    }

    #[test]
    fn a_trust_level_survives_the_endpoint_it_was_learned_on() {
        let table = TrustTable::new(DeviceStorage::memory());
        let did = Did::from_str("did:web:example.com").expect("did");
        let bindings = Bindings::default();
        table.set(did.clone(), Trust::Trusted).expect("assignable");

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
        table
            .set(blocked.clone(), Trust::Blocked)
            .expect("assignable");
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
        assert!(Trust::Blocked < Trust::Anonymous);
        assert!(Trust::Anonymous < Trust::Guest);
        assert!(Trust::Guest < Trust::Trusted);
        assert!(Trust::Trusted < Trust::Myself);
    }
}
