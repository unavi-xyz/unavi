use std::{
    collections::HashMap,
    sync::LazyLock,
};

use anyhow::Context;
use parking_lot::RwLock;
use serde::{
    Deserialize,
    Serialize,
};
use unavi_identity::auth::bindings::Bindings;
use unavi_store::local::{
    Storage,
    decode_hex,
    encode_hex,
};
use xdid::core::did::Did;

/// Transitive trust through the vouch graph.
///
/// Only the local root doc is discoverable today, so no foreign vouch can ever
/// be fetched; this gains real effect once a peer's root doc can be found from
/// its DID.
pub mod vouch;

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
    /// Reachable through someone already trusted.
    Known,
    /// Explicitly trusted.
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

/// Rungs the local user set by hand, keyed by DID.
///
/// Keyed to the DID rather than the endpoint because an `EndpointId` rotates,
/// and a table keyed to one would forget every peer on their next device.
static OVERRIDES: LazyLock<RwLock<HashMap<Did, Trust>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Rungs the vouch graph concluded. Recomputed, never edited, and always lost
/// to an override.
static COMPUTED: LazyLock<RwLock<HashMap<Did, Trust>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// The rung `peer` sits at.
///
/// A peer that has proved no DID cannot rise above [`Trust::Guest`]: an
/// unproven claim is not an identity, so there is nothing to have an opinion
/// about.
#[must_use]
pub fn of_peer(peer: [u8; 32], bindings: &Bindings) -> Trust {
    bindings
        .did_of_bytes(&peer)
        .map_or(Trust::Guest, |did| of_did(&did))
}

/// What the user said, else what the graph worked out, else the default: an
/// override wins outright, so a block takes effect the moment it is set and no
/// rising score can undo it.
#[must_use]
pub fn of_did(did: &Did) -> Trust {
    let manual = OVERRIDES.read().get(did).copied();
    if let Some(trust) = manual {
        return trust;
    }
    COMPUTED.read().get(did).copied().unwrap_or_default()
}

pub fn set_override(did: Did, trust: Trust) {
    OVERRIDES.write().insert(did, trust);
}

pub fn clear_override(did: &Did) {
    OVERRIDES.write().remove(did);
}

/// Weights the local user has vouched at, keyed by DID.
///
/// The plaintext side of what gets published as salted hashes: a voucher knows
/// who it vouched for, and only the voucher does.
static MY_VOUCHES: LazyLock<RwLock<HashMap<Did, u8>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Public, and stable for as long as the vouch list is: rotating it would
/// orphan every hash already published under it.
static SALT: LazyLock<RwLock<[u8; 16]>> = LazyLock::new(|| RwLock::new(rand::random()));

#[must_use]
pub fn salt() -> [u8; 16] {
    *SALT.read()
}

#[must_use]
pub fn my_vouches() -> HashMap<Did, u8> {
    MY_VOUCHES.read().clone()
}

/// Vouches for `did` at `weight`, then re-derives every computed rung.
pub fn add_vouch(me: &Did, did: Did, weight: u8) {
    MY_VOUCHES.write().insert(did, weight.min(100));
    recompute(me, &[]);
}

pub fn remove_vouch(me: &Did, did: &Did) {
    MY_VOUCHES.write().remove(did);
    recompute(me, &[]);
}

/// Rebuilds every computed rung from the local vouches plus `foreign`, the
/// vouches other peers published.
pub fn recompute(me: &Did, foreign: &[(Did, [u8; 16], Vec<vouch::Vouch>)]) {
    let mine = MY_VOUCHES.read().clone();

    let mut graph = vouch::Graph::default();
    for (did, weight) in &mine {
        graph.add(me.clone(), did.clone(), *weight);
    }

    let candidates = mine
        .keys()
        .cloned()
        .chain(foreign.iter().map(|(did, ..)| did.clone()))
        .collect::<Vec<_>>();
    for (voucher, salt, published) in foreign {
        graph.add_published(voucher.clone(), salt, published, &candidates);
    }

    // Scored before the table is taken, so a walk over a large graph does not
    // hold every reader of a rung waiting on it.
    let scored = candidates
        .into_iter()
        .map(|did| {
            let rung = vouch::rung(graph.score(me, &did));
            (did, rung)
        })
        .collect::<HashMap<_, _>>();
    *COMPUTED.write() = scored;
}

/// The table's key in a [`Storage`], and the previous good copy kept beside it.
const TABLE_KEY: &str = "trust.toml";
const BACKUP_KEY: &str = "trust.toml.bak";

#[derive(Default, Serialize, Deserialize)]
struct Stored {
    #[serde(default)]
    salt:    String,
    #[serde(default)]
    peers:   HashMap<String, Trust>,
    #[serde(default)]
    vouches: HashMap<String, u8>,
}

/// Loads the manual rungs from `storage`, discarding entries that no longer
/// parse as DIDs rather than refusing the whole file.
///
/// A table that cannot be read at all is an error rather than an empty start:
/// coming up clean would silently un-block every peer the user ejected. The
/// previous good copy is tried first so a truncated write is survivable.
pub fn load(storage: &Storage) -> anyhow::Result<()> {
    let stored = match storage.read(TABLE_KEY) {
        Ok(Some(text)) => Some(toml::from_str::<Stored>(&text).context("parse trust table")?),
        Ok(None) => None,
        Err(err) => {
            // The backup is only a rescue when it parses; missing or broken are
            // both the same loss as the table itself.
            tracing::warn!(?err, "trust table unreadable, falling back to the backup");
            Some(match storage.read(BACKUP_KEY)? {
                Some(text) => {
                    toml::from_str::<Stored>(&text).context("parse the backup trust table")?
                }
                None => {
                    return Err(err).context("the trust table is unreadable and no backup exists");
                }
            })
        }
    };
    let Some(stored) = stored else {
        return Ok(());
    };

    match hex_to_salt(&stored.salt) {
        Ok(bytes) => *SALT.write() = bytes,
        Err(err) => tracing::warn!(?err, "keeping the session salt"),
    }

    let mut table = OVERRIDES.write();
    for (did, trust) in stored.peers {
        match did.parse::<Did>() {
            Ok(did) => {
                table.insert(did, trust);
            }
            Err(err) => tracing::warn!(?err, %did, "dropping unparseable trust entry"),
        }
    }
    drop(table);

    let mut vouches = MY_VOUCHES.write();
    for (did, weight) in stored.vouches {
        match did.parse::<Did>() {
            Ok(did) => {
                vouches.insert(did, weight.min(100));
            }
            Err(err) => tracing::warn!(?err, %did, "dropping unparseable vouch"),
        }
    }
    drop(vouches);

    Ok(())
}

/// Writes the previous good copy aside, then replaces the table through the
/// backend's atomic write: a trust table half-written by a crash reads as no
/// blocks at all.
pub fn save(storage: &Storage) -> anyhow::Result<()> {
    let stored = Stored {
        salt:    encode_hex(&*SALT.read()),
        peers:   OVERRIDES
            .read()
            .iter()
            .map(|(did, trust)| (did.to_string(), *trust))
            .collect(),
        vouches: MY_VOUCHES
            .read()
            .iter()
            .map(|(did, weight)| (did.to_string(), *weight))
            .collect(),
    };

    let text = toml::to_string_pretty(&stored)?;
    // Whatever is current now becomes the fallback; an unreadable current is
    // replaced rather than carried forward.
    if let Ok(Some(previous)) = storage.read(TABLE_KEY) {
        storage.write(BACKUP_KEY, &previous)?;
    }

    storage.write(TABLE_KEY, &text)
}

/// Operates on bytes: `text.len()` is a byte count, and slicing a `&str` at a
/// byte offset that is not a character boundary panics — on a file the user
/// can edit.
fn hex_to_salt(text: &str) -> anyhow::Result<[u8; 16]> {
    let bytes = decode_hex(text).context("salt is not hex")?;
    bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("salt must be 16 bytes"))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use iroh::{
        EndpointId,
        SecretKey,
    };

    use super::*;

    /// Every test here shares the one table.
    static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn peer() -> EndpointId {
        SecretKey::generate().public()
    }

    fn me() -> Did {
        Did::from_str("did:web:me.example").expect("did")
    }

    /// A fresh table on disk, distinct per test so parallel runs never share a
    /// file, under the shared globals the lock serializes.
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
        assert_eq!(of_peer([3; 32], &Bindings::default()), Trust::Guest);
    }

    #[test]
    fn a_rung_survives_the_endpoint_it_was_learned_on() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        let did = Did::from_str("did:web:example.com").expect("did");
        let bindings = Bindings::default();
        set_override(did.clone(), Trust::Trusted);

        let (first, second) = (peer(), peer());
        bindings.bind(first, did.clone());
        assert_eq!(of_peer(*first.as_bytes(), &bindings), Trust::Trusted);

        bindings.unbind(first);
        bindings.bind(second, did.clone());
        assert_eq!(
            of_peer(*second.as_bytes(), &bindings),
            Trust::Trusted,
            "the same DID on a new endpoint keeps its rung"
        );

        clear_override(&did);
    }

    #[test]
    fn a_users_own_decision_beats_what_the_graph_worked_out() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        let did = Did::from_str("did:web:demoted.example").expect("did");

        add_vouch(&me(), did.clone(), 100);
        assert_eq!(of_did(&did), Trust::Trusted);

        set_override(did.clone(), Trust::Blocked);
        assert_eq!(
            of_did(&did),
            Trust::Blocked,
            "a block must take effect against a peer the graph rates highly"
        );

        clear_override(&did);
        assert_eq!(of_did(&did), Trust::Trusted);
        remove_vouch(&me(), &did);
    }

    #[test]
    fn the_table_survives_a_round_trip_through_disk() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        let (dir, storage) = storage();

        let blocked = Did::from_str("did:web:blocked.example").expect("did");
        set_override(blocked.clone(), Trust::Blocked);
        MY_VOUCHES
            .write()
            .insert(Did::from_str("did:web:vouched.example").expect("did"), 90);
        let before = *SALT.read();
        save(&storage).expect("save");

        OVERRIDES.write().clear();
        MY_VOUCHES.write().clear();
        *SALT.write() = [0; 16];
        load(&storage).expect("load");

        assert_eq!(of_did(&blocked), Trust::Blocked);
        assert_eq!(
            MY_VOUCHES.read().values().copied().next(),
            Some(90),
            "a vouch list round-trips whether or not the graph reads it"
        );
        assert_eq!(
            *SALT.read(),
            before,
            "rotating the salt would orphan every hash already published"
        );

        clear_override(&blocked);
        MY_VOUCHES.write().clear();
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_salt_that_is_not_hex_is_refused_rather_than_panicking() {
        for salt in ["", "aa", "€€€€€€€€€€€", &"z".repeat(32)] {
            assert!(
                hex_to_salt(salt).is_err(),
                "a hand-editable file must not reach a slicing panic"
            );
        }
        assert_eq!(hex_to_salt(&"ab".repeat(16)).expect("hex"), [0xAB; 16]);
    }

    #[test]
    fn a_corrupt_table_is_an_error_not_an_empty_start() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        let (dir, storage) = storage();

        assert!(
            load(&storage).is_ok(),
            "a first run has no table and that is not a failure"
        );

        std::fs::write(dir.join("trust.toml"), "peers = [[[").expect("write");
        assert!(
            load(&storage).is_err(),
            "coming up clean would silently unblock every ejected peer"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_truncated_write_leaves_the_previous_table_readable() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        let (dir, storage) = storage();

        let blocked = Did::from_str("did:web:kept.example").expect("did");
        set_override(blocked.clone(), Trust::Blocked);
        save(&storage).expect("save");
        save(&storage).expect("save again, rotating the table into the backup");

        std::fs::write(dir.join("trust.toml"), "peers = [[[").expect("truncate");
        OVERRIDES.write().clear();
        load(&storage).expect("the backup carries the table");

        assert_eq!(of_did(&blocked), Trust::Blocked);

        clear_override(&blocked);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn the_ladder_is_ordered_from_blocked_up() {
        assert!(Trust::Blocked < Trust::Guest);
        assert!(Trust::Guest < Trust::Known);
        assert!(Trust::Known < Trust::Trusted);
        assert!(Trust::Trusted < Trust::Myself);
    }

    #[test]
    fn a_blocked_peer_clears_nothing() {
        for required in [
            Trust::Blocked,
            Trust::Guest,
            Trust::Known,
            Trust::Trusted,
            Trust::Myself,
        ] {
            assert!(
                !Trust::Blocked.clears(required),
                "blocked must not clear {required:?}"
            );
        }
    }

    #[test]
    fn the_default_rung_clears_the_open_default() {
        assert!(Trust::default().clears(Trust::Guest));
        assert!(!Trust::Guest.clears(Trust::Known));
        assert!(Trust::Myself.clears(Trust::Trusted));
    }
}
