use std::sync::LazyLock;

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
/// desktop, no screen pointer in VR — which is what `active` reports as false.
pub static POINTER_REGISTRY: LazyLock<RwLock<[Option<Pointer>; PointerKind::COUNT]>> =
    LazyLock::new(|| RwLock::new([None; PointerKind::COUNT]));

pub fn snapshot_pointers(
    pointers: Query<(&PointerAnchor, &GlobalTransform, &PointerInteraction)>,
    state: Res<ActionState>,
) {
    let mut snapshot = [None; PointerKind::COUNT];

    for (anchor, transform, interaction) in pointers {
        let kind = anchor.0;
        snapshot[kind.index()] = Some(Pointer {
            id:     kind.into(),
            active: true,
            ray:    ray_of(transform).into(),
            grasp:  state.value(Action::Grab(kind)),
            axis:   axis_of(kind, &state),
            hit:    nearest_hit(interaction).map(Into::into),
        });
    }

    *POINTER_REGISTRY.write() = snapshot;
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
