use std::sync::Arc;

use bevy::{
    picking::pointer::PointerInteraction,
    prelude::*,
};
use parking_lot::RwLock;
use unavi_input::{
    action::{
        Action,
        ActionState,
    },
    pointer::{
        PointerAnchor,
        PointerKind,
        nearest_hit,
        ray_of,
    },
};

use crate::runtime::shared::wired::input::types::Pointer;

/// This frame's pointer state, for scripts to read off-thread. A script asking
/// where the user's hands are wants this frame's answer, not one that reached
/// it through a queue.
///
/// A pointer with no entry is one the rig never spawned — no hand tracking on
/// desktop, no screen pointer in VR — which is what [`Pointers::all`] reports
/// as inactive.
#[derive(Resource, Clone, Default)]
pub struct Pointers(Arc<RwLock<[Option<Pointer>; PointerKind::COUNT]>>);

impl Pointers {
    pub fn store(&self, snapshot: [Option<Pointer>; PointerKind::COUNT]) {
        *self.0.write() = snapshot;
    }

    #[must_use]
    pub fn all(&self) -> Vec<Pointer> {
        let snapshot = self.0.read();
        PointerKind::ALL
            .into_iter()
            .map(|kind| snapshot[kind.index()].unwrap_or_else(|| Pointer::inactive(kind)))
            .collect()
    }
}

pub fn snapshot_pointers(
    pointers: Query<(&PointerAnchor, &GlobalTransform, &PointerInteraction)>,
    state: Res<ActionState>,
    registry: Res<Pointers>,
) {
    let mut snapshot = [None; PointerKind::COUNT];

    for (anchor, transform, interaction) in pointers {
        let kind = anchor.0;
        snapshot[kind.index()] = Some(Pointer {
            kind,
            active: true,
            ray: ray_of(transform).into(),
            trigger: state.value(Action::Trigger(kind)),
            grip: state.value(Action::Grip(kind)),
            axis: axis_of(kind, &state),
            hit: nearest_hit(interaction).map(Into::into),
        });
    }

    registry.store(snapshot);
}

/// The stick that steers with this pointer's hand. A screen pointer has none:
/// its look axis is a mouse delta, not a held direction.
const fn axis_of(kind: PointerKind, state: &ActionState) -> Vec2 {
    match kind {
        PointerKind::Screen => Vec2::ZERO,
        PointerKind::LeftHand => state.axis(Action::Move),
        PointerKind::RightHand => state.axis(Action::Look),
    }
}
