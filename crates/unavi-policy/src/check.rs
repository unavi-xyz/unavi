use std::sync::{
    Arc,
    RwLock,
};

use hsd::id::DocId;
use unavi_identity::auth::bindings::Bindings;

use crate::{
    error::PolicyError,
    reach::{
        Reach,
        permits,
        same_owner,
    },
    registry,
    tier::Tier,
    trust::{
        self,
        Trust,
    },
};

/// What policy needs from the networking layer, installed rather than
/// depended upon.
///
/// Keeps every "may X do Y to Z" here without dragging the networking stack
/// into the script sandbox.
#[derive(Clone)]
pub struct Resolver {
    /// The DIDs peers have proven over `wired/auth`.
    pub bindings:  Arc<Bindings>,
    /// The peer whose pin owns `doc` within `space`.
    pub owner:     fn(space: DocId, doc: DocId) -> Option<[u8; 32]>,
    /// The space some peer's pin places `doc` in. `Some` means the document
    /// arrived over the network rather than being minted here.
    pub space_of:  fn(doc: DocId) -> Option<DocId>,
    pub self_peer: fn() -> Option<[u8; 32]>,
}

static RESOLVER: RwLock<Option<Resolver>> = RwLock::new(None);

pub fn set_resolver(resolver: Resolver) {
    *RESOLVER.write().expect("resolver lock") = Some(resolver);
}

fn resolver() -> Option<Resolver> {
    RESOLVER.read().expect("resolver lock").clone()
}

/// Where a document stands: what it demands of writers, which space it is in,
/// and who authored it. Resolved once per check; the write path runs on every
/// prim write.
struct Standing {
    reach: Reach,
    space: Option<DocId>,
    owner: Option<[u8; 32]>,
}

fn standing(doc: DocId) -> Standing {
    let root = registry::root(doc);
    let record = registry::get(doc);
    let resolver = resolver();

    let replicated = resolver.as_ref().and_then(|r| (r.space_of)(root));
    let space = registry::registered_space(root).or(replicated);

    let owner = space
        .zip(resolver.as_ref())
        .and_then(|(space, r)| (r.owner)(space, root))
        .or_else(|| {
            // Nothing pins the root and it is absent from the replica index, so
            // it was minted here. A document that *is* in the index arrived
            // from a peer, and must never fall back to reading as local.
            (replicated.is_none())
                .then(|| resolver.as_ref().and_then(|r| (r.self_peer)()))
                .flatten()
        });

    Standing {
        reach: record.reach,
        space,
        owner,
    }
}

/// The space `doc` belongs to.
///
/// Either the space it was registered into, or — for a pinned document, which
/// is namespace-backed and has no local registration — the space some peer's
/// pin names. A prefab instance answers with its host's, since it has neither
/// of its own.
#[must_use]
pub fn space_of(doc: DocId) -> Option<DocId> {
    let root = registry::root(doc);
    registry::registered_space(root).or_else(|| resolver().and_then(|r| (r.space_of)(root)))
}

#[must_use]
pub fn same_space(a: DocId, b: DocId) -> bool {
    match (space_of(a), space_of(b)) {
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}

/// The rung to judge a document by, given the peer that owns it.
///
/// A document nobody can be shown to have authored is judged as a stranger's,
/// never as the local user's: the fail-open direction here is what decides
/// whether unattributable content can write everything you own.
#[must_use]
pub fn trust_of(owner: Option<[u8; 32]>) -> Trust {
    let Some(peer) = owner else {
        return Trust::Guest;
    };
    let Some(resolver) = resolver() else {
        return Trust::Guest;
    };
    if (resolver.self_peer)() == Some(peer) {
        Trust::Myself
    } else {
        trust::of_peer(peer, &resolver.bindings)
    }
}

/// The tier `doc` was loaded at.
#[must_use]
pub fn tier_of(doc: DocId) -> Tier {
    registry::get(doc).policy.tier
}

/// Whether `caller` may write `target`.
///
/// Judged through the owners: a document's authority is a function of the
/// trust its owning peer has, so there is no per-document matrix. The caller's
/// tier is read here rather than passed in — a caller handing over its own
/// tier could hand over the wrong one.
pub fn write(caller: DocId, target: DocId) -> Result<(), PolicyError> {
    if caller == target {
        return Ok(());
    }
    let caller_standing = standing(caller);
    let target_standing = standing(target);

    let co_present = tier_of(caller).crosses_space_boundaries()
        || (caller_standing.space.is_some() && caller_standing.space == target_standing.space);

    permits(
        trust_of(caller_standing.owner),
        target_standing.reach,
        same_owner(caller_standing.owner, target_standing.owner),
        co_present,
    )
}

/// Whether `caller` may read `target`.
///
/// Reads are open within a space, so membership is the whole gate; a document
/// that wants to be unreadable has to live in a namespace the reader has no id
/// for.
pub fn read(caller: DocId, target: DocId) -> Result<(), PolicyError> {
    if caller == target || tier_of(caller).crosses_space_boundaries() || same_space(caller, target)
    {
        Ok(())
    } else {
        Err(PolicyError::NotCoPresent)
    }
}

/// Whether a document is placed well enough to reach anything outside itself.
///
/// A document outside every space has no co-presence to appeal to, so its only
/// possible reach is same-owner; refusing up front stops unattributable
/// content riding an ownership answer it did not earn.
pub fn placed(caller: DocId) -> Result<(), PolicyError> {
    if tier_of(caller).crosses_space_boundaries() || space_of(caller).is_some() {
        Ok(())
    } else {
        Err(PolicyError::NotCoPresent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::DocumentPolicy;

    /// Every test here shares the one registry and the one resolver.
    static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    const ME: [u8; 32] = [1; 32];
    const PEER: [u8; 32] = [2; 32];

    const SPACE: DocId = DocId([10; 32]);
    const MINE: DocId = DocId([11; 32]);
    const THEIRS: DocId = DocId([12; 32]);
    const SHELL: DocId = DocId([13; 32]);
    const THEIR_INSTANCE: DocId = DocId([14; 32]);
    const TOOL: DocId = DocId([15; 32]);
    const ORPHAN: DocId = DocId([16; 32]);

    fn owner(space: DocId, doc: DocId) -> Option<[u8; 32]> {
        if space != SPACE {
            return None;
        }
        if doc == MINE {
            Some(ME)
        } else if doc == THEIRS {
            Some(PEER)
        } else {
            None
        }
    }

    fn space_of(doc: DocId) -> Option<DocId> {
        (doc == MINE || doc == THEIRS).then_some(SPACE)
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "signature is fixed by Resolver::self_peer"
    )]
    fn self_peer() -> Option<[u8; 32]> {
        Some(ME)
    }

    /// One space, one document each peer pins in it, a shell and a tool
    /// standing outside every space, and a prefab the peer's document composed.
    fn scene() {
        set_resolver(Resolver {
            bindings: Arc::default(),
            owner,
            space_of,
            self_peer,
        });
        registry::update(SPACE, |r| {
            r.space = Some(SPACE);
            r.policy = DocumentPolicy::space();
        });
        for doc in [SHELL, TOOL] {
            registry::update(doc, |r| {
                r.policy = DocumentPolicy::system();
                r.reach = Reach::own_only();
            });
        }
        registry::update(THEIR_INSTANCE, |r| r.host = Some(THEIRS));
        registry::update(ORPHAN, |r| r.policy = DocumentPolicy::untrusted());
    }

    fn teardown() {
        for doc in [SPACE, MINE, THEIRS, SHELL, TOOL, THEIR_INSTANCE, ORPHAN] {
            registry::forget(doc);
        }
    }

    #[test]
    fn a_peers_prefab_instance_is_attributed_to_the_peer_not_to_you() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        scene();

        assert_eq!(
            standing(THEIR_INSTANCE).owner,
            Some(PEER),
            "an instance is never pinned, so it has to resolve through its host"
        );
        assert_eq!(trust_of(standing(THEIR_INSTANCE).owner), Trust::Guest);

        teardown();
    }

    #[test]
    fn a_peers_instance_cannot_reach_the_shell() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        scene();

        assert_eq!(
            write(THEIR_INSTANCE, SHELL),
            Err(PolicyError::NotCoPresent),
            "content a peer brought must not write the shell it is standing in"
        );

        teardown();
    }

    #[test]
    fn the_shell_and_its_tools_still_reach_each_other() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        scene();

        assert_eq!(
            write(TOOL, SHELL),
            Ok(()),
            "both are locally authored, so same-owner answers before the rung"
        );

        teardown();
    }

    #[test]
    fn a_stranger_may_still_write_open_content_beside_it() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        scene();

        assert_eq!(
            write(THEIRS, MINE),
            Ok(()),
            "co-present peers writing each other's open props is the default"
        );

        teardown();
    }

    #[test]
    fn an_unplaced_document_writes_nothing_it_does_not_own() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        scene();

        assert!(placed(ORPHAN).is_err());
        assert!(placed(SHELL).is_ok());
        assert!(placed(MINE).is_ok());

        teardown();
    }

    #[test]
    fn reads_are_open_within_a_space_and_closed_across_one() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        scene();

        assert_eq!(read(THEIRS, MINE), Ok(()));
        assert_eq!(read(THEIRS, SHELL), Err(PolicyError::NotCoPresent));
        assert_eq!(read(SHELL, THEIRS), Ok(()));

        teardown();
    }
}
