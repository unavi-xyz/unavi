use bevy::prelude::*;
use unavi_input::pointer::PointerKind;

use crate::runtime::{
    native::wired::input::bindings::wired::{
        input::types::{
            Hit,
            InputAction,
            InputEvent,
            Pointer,
            PointerKind as WitPointerKind,
            Ray,
        },
        math::types::{
            Vec2 as WitVec2,
            Vec3 as WitVec3,
        },
    },
    shared::wired::input::types as shared_types,
};

impl From<Vec2> for WitVec2 {
    fn from(v: Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<Vec3> for WitVec3 {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<PointerKind> for WitPointerKind {
    fn from(kind: PointerKind) -> Self {
        match kind {
            PointerKind::Screen => Self::Screen,
            PointerKind::LeftHand => Self::LeftHand,
            PointerKind::RightHand => Self::RightHand,
        }
    }
}

impl From<WitPointerKind> for PointerKind {
    fn from(kind: WitPointerKind) -> Self {
        match kind {
            WitPointerKind::Screen => Self::Screen,
            WitPointerKind::LeftHand => Self::LeftHand,
            WitPointerKind::RightHand => Self::RightHand,
        }
    }
}

impl From<shared_types::Ray> for Ray {
    fn from(ray: shared_types::Ray) -> Self {
        Self {
            origin: ray.origin.into(),
            dir:    ray.dir.into(),
        }
    }
}

impl From<shared_types::Hit> for Hit {
    fn from(hit: shared_types::Hit) -> Self {
        Self {
            position: hit.position.into(),
            normal:   hit.normal.into(),
            distance: hit.distance,
        }
    }
}

impl From<shared_types::InputAction> for InputAction {
    fn from(action: shared_types::InputAction) -> Self {
        match action {
            shared_types::InputAction::Press => Self::Press,
            shared_types::InputAction::Release => Self::Release,
            shared_types::InputAction::GripPress => Self::GripPress,
            shared_types::InputAction::GripRelease => Self::GripRelease,
            shared_types::InputAction::Scroll(delta) => Self::Scroll(delta.into()),
            shared_types::InputAction::Enter => Self::Enter,
            shared_types::InputAction::Leave => Self::Leave,
            shared_types::InputAction::MenuPress => Self::MenuPress,
            shared_types::InputAction::MenuRelease => Self::MenuRelease,
        }
    }
}

impl From<shared_types::InputEvent> for InputEvent {
    fn from(event: shared_types::InputEvent) -> Self {
        Self {
            pointer: event.pointer.into(),
            action:  event.action.into(),
            ray:     event.ray.into(),
            hit:     event.hit.map(Into::into),
        }
    }
}

impl From<shared_types::Pointer> for Pointer {
    fn from(pointer: shared_types::Pointer) -> Self {
        Self {
            kind:    pointer.kind.into(),
            active:  pointer.active,
            ray:     pointer.ray.into(),
            trigger: pointer.trigger,
            grip:    pointer.grip,
            axis:    pointer.axis.into(),
            hit:     pointer.hit.map(Into::into),
        }
    }
}
