use std::{
    collections::HashMap,
    fmt::Write,
    path::PathBuf,
    sync::LazyLock,
};

use anyhow::Context;
use parking_lot::RwLock;
use serde::{
    Deserialize,
    Serialize,
};
use xdid::core::did::Did;

use crate::identity;

/// Transitive trust through the vouch graph.
///
/// The metric walks whatever edges it is given, but only the local root doc is
/// discoverable today, so no foreign vouch can ever be fetched and the graph
/// can conclude nothing a direct override could not. It gains real effect once
/// a peer's root doc can be found from its DID.
pub mod vouch;

/// How much a peer is trusted, as one ordinal rung.
///
/// Ordinal rather than a capability matrix because the granularity users
/// actually manage is per-rung: every capability names the minimum rung it
/// needs, and a user moves peers between rungs instead of editing a grid.
///
/// The opinion is the local viewer's and is never gossiped as authoritative,
/// so there is nothing here for a peer or a space to spoof. Ranks *peers*, not
/// documents — [`crate::tier::Tier`] is the document side.
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
pub fn of_peer(peer: [u8; 32]) -> Trust {
    identity::did_of(peer).map_or(Trust::Guest, |did| of_did(&did))
}

/// What the user said, else what the graph worked out, else the default.
///
/// The user's own decision wins outright, which is what makes demotion
/// instant: a block takes effect the moment it is set, and no rising score can
/// undo it.
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
pub fn add_vouch(did: Did, weight: u8) {
    MY_VOUCHES.write().insert(did, weight.min(100));
    recompute(&[]);
}

pub fn remove_vouch(did: &Did) {
    MY_VOUCHES.write().remove(did);
    recompute(&[]);
}

/// Rebuilds every computed rung from the local vouches plus `foreign`, the
/// vouches other peers published.
pub fn recompute(foreign: &[(Did, [u8; 16], Vec<vouch::Vouch>)]) {
    let Some(me) = identity::self_did() else {
        COMPUTED.write().clear();
        return;
    };
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
            let rung = vouch::rung(graph.score(&me, &did));
            (did, rung)
        })
        .collect::<HashMap<_, _>>();
    *COMPUTED.write() = scored;
}

#[derive(Default, Serialize, Deserialize)]
struct Stored {
    #[serde(default)]
    salt:    String,
    #[serde(default)]
    peers:   HashMap<String, Trust>,
    #[serde(default)]
    vouches: HashMap<String, u8>,
}

/// Loads the manual rungs from `dir`, discarding entries that no longer parse
/// as DIDs rather than refusing the whole file.
///
/// A table that cannot be read at all is an error rather than an empty start:
/// coming up clean would silently un-block every peer the user ejected, which
/// is the one direction this file must never fail in. The previous good copy
/// is tried first so a truncated write is survivable without a prompt.
pub fn load(dir: &std::path::Path) -> anyhow::Result<()> {
    let stored = match read_table(&table_path(dir)) {
        Ok(stored) => stored,
        Err(err) => {
            tracing::warn!(?err, "trust table unreadable, falling back to the backup");
            // An absent backup is fatal here, not a clean first run: the table
            // itself exists and could not be read, and coming up empty is the
            // failure this whole path is guarding against.
            Some(
                read_table(&backup_path(dir))
                    .ok()
                    .flatten()
                    .ok_or(err)
                    .context("the trust table is unreadable and has no usable backup")?,
            )
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

    recompute(&[]);
    Ok(())
}

/// `Ok(None)` when there is no file yet, which is a first run rather than a
/// failure.
fn read_table(path: &std::path::Path) -> anyhow::Result<Option<Stored>> {
    match std::fs::read_to_string(path) {
        Ok(contents) => Ok(Some(toml::from_str::<Stored>(&contents)?)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.into()),
    }
}

/// Writes through a temporary file and renames over the table.
///
/// The previous copy is kept: a trust table half-written by a crash reads as no
/// blocks at all, so the write has to be atomic and the last good copy survive.
pub fn save(dir: &std::path::Path) -> anyhow::Result<()> {
    let stored = Stored {
        salt:    hex(&*SALT.read()),
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

    let path = table_path(dir);
    let temp = dir.join("trust.toml.tmp");
    std::fs::write(&temp, toml::to_string_pretty(&stored)?)?;
    if path.exists() {
        std::fs::rename(&path, backup_path(dir))?;
    }
    std::fs::rename(&temp, &path)?;
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut out, b| {
        let _ = write!(out, "{b:02x}");
        out
    })
}

/// Operates on bytes: `hex.len()` is a byte count, and slicing a `&str` at a
/// byte offset that is not a character boundary panics — on a file the user
/// can edit.
fn hex_to_salt(hex: &str) -> anyhow::Result<[u8; 16]> {
    let bytes = hex.as_bytes();
    if bytes.len() != 32 {
        anyhow::bail!("salt must be 16 bytes")
    }
    let mut out = [0u8; 16];
    for (byte, pair) in out.iter_mut().zip(bytes.as_chunks::<2>().0) {
        let pair = std::str::from_utf8(pair).context("salt is not hex")?;
        *byte = u8::from_str_radix(pair, 16)?;
    }
    Ok(out)
}

fn table_path(dir: &std::path::Path) -> PathBuf {
    dir.join("trust.toml")
}

fn backup_path(dir: &std::path::Path) -> PathBuf {
    dir.join("trust.toml.bak")
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    /// Every test here shares the one table.
    static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn an_unproven_peer_is_a_guest() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        assert_eq!(of_peer([3; 32]), Trust::Guest);
    }

    #[test]
    fn a_rung_survives_the_endpoint_it_was_learned_on() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        let did = Did::from_str("did:web:example.com").expect("did");
        set_override(did.clone(), Trust::Trusted);

        identity::bind([1; 32], did.clone());
        assert_eq!(of_peer([1; 32]), Trust::Trusted);

        identity::unbind([1; 32]);
        identity::bind([2; 32], did);
        assert_eq!(
            of_peer([2; 32]),
            Trust::Trusted,
            "the same DID on a new endpoint keeps its rung"
        );

        identity::unbind([2; 32]);
    }

    #[test]
    fn a_users_own_decision_beats_what_the_graph_worked_out() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        let did = Did::from_str("did:web:demoted.example").expect("did");
        identity::set_self(Did::from_str("did:web:me.example").expect("did"));

        add_vouch(did.clone(), 100);
        assert_eq!(of_did(&did), Trust::Trusted);

        set_override(did.clone(), Trust::Blocked);
        assert_eq!(
            of_did(&did),
            Trust::Blocked,
            "a block must take effect against a peer the graph rates highly"
        );

        clear_override(&did);
        assert_eq!(of_did(&did), Trust::Trusted);
        remove_vouch(&did);
    }

    #[test]
    fn the_table_survives_a_round_trip_through_disk() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        let dir = std::env::temp_dir().join(format!("unavi-trust-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");

        let blocked = Did::from_str("did:web:blocked.example").expect("did");
        identity::set_self(Did::from_str("did:web:me.example").expect("did"));
        set_override(blocked.clone(), Trust::Blocked);
        MY_VOUCHES
            .write()
            .insert(Did::from_str("did:web:vouched.example").expect("did"), 90);
        let before = *SALT.read();
        save(&dir).expect("save");

        OVERRIDES.write().clear();
        MY_VOUCHES.write().clear();
        *SALT.write() = [0; 16];
        load(&dir).expect("load");

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
        let dir = std::env::temp_dir().join(format!("unavi-trust-corrupt-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");

        assert!(
            load(&dir).is_ok(),
            "a first run has no table and that is not a failure"
        );

        std::fs::write(dir.join("trust.toml"), "peers = [[[").expect("write");
        assert!(
            load(&dir).is_err(),
            "coming up clean would silently unblock every ejected peer"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_truncated_write_leaves_the_previous_table_readable() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        let dir = std::env::temp_dir().join(format!("unavi-trust-backup-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");

        let blocked = Did::from_str("did:web:kept.example").expect("did");
        set_override(blocked.clone(), Trust::Blocked);
        save(&dir).expect("save");
        save(&dir).expect("save again, rotating the table into the backup");

        std::fs::write(dir.join("trust.toml"), "peers = [[[").expect("truncate");
        OVERRIDES.write().clear();
        load(&dir).expect("the backup carries the table");

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
