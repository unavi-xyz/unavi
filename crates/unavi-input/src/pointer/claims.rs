use std::sync::LazyLock;

use parking_lot::RwLock;

use crate::pointer::PointerKind;

/// Who holds a pointer, identified by their document.
///
/// An owner rather than a flag, because a claim is a lease and a lease with no
/// holder cannot be reclaimed: a script that traps while holding a pointer
/// would keep it until the process exits, and the pointer it keeps is one of
/// the user's hands.
pub type Owner = [u8; 32];

/// Pointers a script has taken over. While one is claimed the host runs none
/// of its own interactions for it — no grab, no crosshair — so an equipped
/// tool and the host cannot both answer one press.
///
/// A static rather than a resource because a script claims from off the main
/// thread, the same reason the transform snapshots a script reads are one.
static CLAIMED: LazyLock<RwLock<[Option<Owner>; PointerKind::COUNT]>> =
    LazyLock::new(|| RwLock::new([None; PointerKind::COUNT]));

#[must_use]
pub fn is_claimed(kind: PointerKind) -> bool {
    CLAIMED.read()[kind.index()].is_some()
}

#[must_use]
pub fn owner(kind: PointerKind) -> Option<Owner> {
    CLAIMED.read()[kind.index()]
}

/// Exclusive and first-come; `false` means someone else holds it.
#[must_use]
pub fn claim(kind: PointerKind, owner: Owner) -> bool {
    let mut claimed = CLAIMED.write();
    if claimed[kind.index()].is_some() {
        return false;
    }
    claimed[kind.index()] = Some(owner);
    true
}

pub fn release(kind: PointerKind) {
    CLAIMED.write()[kind.index()] = None;
}

/// Releases every pointer `owner` holds, returning how many came back.
///
/// The break-out the lease exists for: a document that goes away — unloaded,
/// trapped, or despawned — cannot be relied on to give its claims back itself.
pub fn release_all_of(owner: Owner) -> usize {
    let mut claimed = CLAIMED.write();
    let mut released = 0;
    for slot in claimed.iter_mut() {
        if *slot == Some(owner) {
            *slot = None;
            released += 1;
        }
    }
    drop(claimed);
    released
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALICE: Owner = [1; 32];
    const BOB: Owner = [2; 32];

    /// Every test here shares one static, so each uses a pointer of its own.
    #[test]
    fn a_claim_is_exclusive_and_releasable() {
        let kind = PointerKind::RightHand;
        assert!(claim(kind, ALICE));
        assert!(!claim(kind, BOB), "second claim loses");
        assert!(is_claimed(kind));
        assert_eq!(owner(kind), Some(ALICE));

        release(kind);
        assert!(!is_claimed(kind));
    }

    #[test]
    fn a_claim_does_not_spread_to_the_other_hand() {
        assert!(claim(PointerKind::LeftHand, ALICE));
        assert!(!is_claimed(PointerKind::Screen));
        release(PointerKind::LeftHand);
    }

    #[test]
    fn a_departing_owner_gets_every_pointer_it_held_taken_back() {
        assert!(claim(PointerKind::Screen, ALICE));

        assert_eq!(
            release_all_of(BOB),
            0,
            "another owner's claim is not theirs"
        );
        assert!(is_claimed(PointerKind::Screen));

        assert_eq!(release_all_of(ALICE), 1);
        assert!(
            !is_claimed(PointerKind::Screen),
            "a trapped script must not keep one of the user's hands"
        );
    }
}
