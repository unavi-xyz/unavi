use std::sync::LazyLock;

use parking_lot::RwLock;

use crate::pointer::PointerKind;

/// Pointers a script has taken over. While one is claimed the host runs none
/// of its own interactions for it — no grab, no crosshair — so an equipped
/// tool and the host cannot both answer one press.
///
/// A static rather than a resource because a script claims from off the main
/// thread, the same reason the transform snapshots a script reads are one.
static CLAIMED: LazyLock<RwLock<[bool; PointerKind::COUNT]>> =
    LazyLock::new(|| RwLock::new([false; PointerKind::COUNT]));

#[must_use]
pub fn is_claimed(kind: PointerKind) -> bool {
    CLAIMED.read()[kind.index()]
}

/// Exclusive and first-come; `false` means someone else holds it.
#[must_use]
pub fn claim(kind: PointerKind) -> bool {
    let mut claimed = CLAIMED.write();
    if claimed[kind.index()] {
        return false;
    }
    claimed[kind.index()] = true;
    true
}

pub fn release(kind: PointerKind) {
    CLAIMED.write()[kind.index()] = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every test here shares one static, so each uses a pointer of its own.
    #[test]
    fn a_claim_is_exclusive_and_releasable() {
        let kind = PointerKind::RightHand;
        assert!(claim(kind));
        assert!(!claim(kind), "second claim loses");
        assert!(is_claimed(kind));

        release(kind);
        assert!(!is_claimed(kind));
    }

    #[test]
    fn a_claim_does_not_spread_to_the_other_hand() {
        assert!(claim(PointerKind::LeftHand));
        assert!(!is_claimed(PointerKind::Screen));
        release(PointerKind::LeftHand);
    }
}
