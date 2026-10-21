//! Controls taken from the host, per pointer.

use std::sync::{
    Arc,
    atomic::{
        AtomicU32,
        Ordering,
    },
};

use bevy::prelude::*;

use crate::pointer::PointerKind;

/// A control the host acts on by itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    /// Squeezing picks up what the pointer is on.
    Grip,
    /// A hand's stick moves or turns the user.
    Stick,
}

impl Control {
    const COUNT: usize = 2;

    const fn index(self) -> usize {
        match self {
            Self::Grip => 0,
            Self::Stick => 1,
        }
    }
}

const SLOTS: usize = PointerKind::COUNT * Control::COUNT;

/// How many [`Claim`]s hold each pointer's controls. The host leaves a
/// claimed control alone. Cloning shares the counts.
#[derive(Resource, Clone, Default)]
pub struct Claims(Arc<[AtomicU32; SLOTS]>);

impl Claims {
    fn slot(&self, kind: PointerKind, control: Control) -> &AtomicU32 {
        &self.0[kind.index() * Control::COUNT + control.index()]
    }

    /// Holds `control` of `kind` until the claim is dropped.
    #[must_use]
    pub fn take(&self, kind: PointerKind, control: Control) -> Claim {
        self.slot(kind, control).fetch_add(1, Ordering::Relaxed);
        Claim {
            claims: self.clone(),
            kind,
            control,
        }
    }

    #[must_use]
    pub fn is_claimed(&self, kind: PointerKind, control: Control) -> bool {
        self.slot(kind, control).load(Ordering::Relaxed) > 0
    }
}

/// One hold on a pointer's control. Dropping it lets go.
pub struct Claim {
    claims:  Claims,
    kind:    PointerKind,
    control: Control,
}

impl Drop for Claim {
    fn drop(&mut self) {
        self.claims
            .slot(self.kind, self.control)
            .fetch_sub(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_control_returns_only_when_its_last_claim_drops() {
        let claims = Claims::default();
        let first = claims.take(PointerKind::RightHand, Control::Grip);
        let second = claims.take(PointerKind::RightHand, Control::Grip);

        drop(first);
        assert!(
            claims.is_claimed(PointerKind::RightHand, Control::Grip),
            "a second claim still holds the grip"
        );
        drop(second);
        assert!(!claims.is_claimed(PointerKind::RightHand, Control::Grip));
    }

    #[test]
    fn a_claim_holds_only_its_own_pointer_and_control() {
        let claims = Claims::default();
        let _claim = claims.take(PointerKind::RightHand, Control::Stick);

        assert!(!claims.is_claimed(PointerKind::LeftHand, Control::Stick));
        assert!(!claims.is_claimed(PointerKind::RightHand, Control::Grip));
    }
}
