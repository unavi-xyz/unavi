use bevy::prelude::*;

use crate::runtime::{
    native::wired::input::bindings::wired::{
        input::types::{
            Hit,
            InputAction,
            InputEvent,
            Pointer,
            PointerId,
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

impl From<shared_types::PointerId> for PointerId {
    fn from(id: shared_types::PointerId) -> Self {
        match id {
            shared_types::PointerId::Screen => Self::Screen,
            shared_types::PointerId::LeftHand => Self::LeftHand,
            shared_types::PointerId::RightHand => Self::RightHand,
        }
    }
}

impl From<PointerId> for shared_types::PointerId {
    fn from(id: PointerId) -> Self {
        match id {
            PointerId::Screen => Self::Screen,
            PointerId::LeftHand => Self::LeftHand,
            PointerId::RightHand => Self::RightHand,
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
            id:     pointer.id.into(),
            active: pointer.active,
            ray:    pointer.ray.into(),
            grasp:  pointer.grasp,
            axis:   pointer.axis.into(),
            hit:    pointer.hit.map(Into::into),
        }
    }
}
