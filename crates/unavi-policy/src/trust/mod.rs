use std::{
    collections::HashMap,
    path::PathBuf,
    sync::LazyLock,
};

use parking_lot::RwLock;
use serde::{
    Deserialize,
    Serialize,
};
use xdid::core::did::Did;

use crate::identity;

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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
    let computed = COMPUTED.read().get(did).copied();
    computed.unwrap_or_default()
}

pub fn set_override(did: Did, trust: Trust) {
    OVERRIDES.write().insert(did, trust);
}

pub fn clear_override(did: &Did) {
    OVERRIDES.write().remove(did);
}

#[must_use]
pub fn overrides() -> HashMap<Did, Trust> {
    OVERRIDES.read().clone()
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
///
/// `foreign` is empty until a peer's root doc can be found from its DID, so
/// today only direct vouches produce a rung. The metric is the same either
/// way — it simply has one hop of graph to walk.
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
        .chain(foreign.iter().map(|(did, _, _)| did.clone()))
        .collect::<Vec<_>>();
    for (voucher, salt, published) in foreign {
        graph.add_published(voucher.clone(), salt, published, &candidates);
    }

    let mut computed = COMPUTED.write();
    computed.clear();
    for did in candidates {
        let score = graph.score(&me, &did);
        computed.insert(did, vouch::rung(score));
    }
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
pub fn load(dir: &std::path::Path) {
    let path = table_path(dir);
    let Ok(contents) = std::fs::read_to_string(&path) else {
        return;
    };
    let stored = match toml::from_str::<Stored>(&contents) {
        Ok(stored) => stored,
        Err(err) => {
            tracing::warn!(?err, "failed to parse trust table, starting empty");
            return;
        }
    };

    if let Ok(bytes) = hex_to_salt(&stored.salt) {
        *SALT.write() = bytes;
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
}

pub fn save(dir: &std::path::Path) -> anyhow::Result<()> {
    let stored = Stored {
        salt:    salt().iter().fold(String::new(), |mut out, b| {
            use std::fmt::Write;
            let _ = write!(out, "{b:02x}");
            out
        }),
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
    std::fs::write(table_path(dir), toml::to_string_pretty(&stored)?)?;
    Ok(())
}

fn hex_to_salt(hex: &str) -> anyhow::Result<[u8; 16]> {
    if hex.len() != 32 {
        anyhow::bail!("salt must be 16 bytes")
    }
    let mut out = [0u8; 16];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)?;
    }
    Ok(out)
}

fn table_path(dir: &std::path::Path) -> PathBuf {
    dir.join("trust.toml")
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
        let vouched = Did::from_str("did:web:vouched.example").expect("did");
        identity::set_self(Did::from_str("did:web:me.example").expect("did"));
        set_override(blocked.clone(), Trust::Blocked);
        add_vouch(vouched.clone(), 90);
        let before = salt();
        save(&dir).expect("save");

        OVERRIDES.write().clear();
        MY_VOUCHES.write().clear();
        *SALT.write() = [0; 16];
        load(&dir);

        assert_eq!(of_did(&blocked), Trust::Blocked);
        assert_eq!(my_vouches().get(&vouched).copied(), Some(90));
        assert_eq!(
            salt(),
            before,
            "rotating the salt would orphan every hash already published"
        );

        clear_override(&blocked);
        remove_vouch(&vouched);
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
