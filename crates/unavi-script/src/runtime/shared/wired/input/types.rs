use bevy::prelude::*;
use unavi_input::pointer::{
    PointerHit,
    PointerKind,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerId {
    Screen,
    LeftHand,
    RightHand,
}

impl From<PointerKind> for PointerId {
    fn from(kind: PointerKind) -> Self {
        match kind {
            PointerKind::Screen => Self::Screen,
            PointerKind::LeftHand => Self::LeftHand,
            PointerKind::RightHand => Self::RightHand,
        }
    }
}

impl From<PointerId> for PointerKind {
    fn from(id: PointerId) -> Self {
        match id {
            PointerId::Screen => Self::Screen,
            PointerId::LeftHand => Self::LeftHand,
            PointerId::RightHand => Self::RightHand,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub origin: Vec3,
    pub dir:    Vec3,
}

impl From<Ray3d> for Ray {
    fn from(ray: Ray3d) -> Self {
        Self {
            origin: ray.origin,
            dir:    *ray.direction,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub position: Vec3,
    pub normal:   Vec3,
    pub distance: f32,
}

impl From<PointerHit> for Hit {
    fn from(hit: PointerHit) -> Self {
        Self {
            position: hit.position,
            normal:   hit.normal,
            distance: hit.distance,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pointer {
    pub id:     PointerId,
    pub active: bool,
    pub ray:    Ray,
    pub grasp:  f32,
    pub axis:   Vec2,
    pub hit:    Option<Hit>,
}

impl Pointer {
    /// A pointer the rig never spawned: no hands on desktop, no screen
    /// pointer in VR. Still listed, so a script can see it is not there.
    #[must_use]
    pub fn inactive(kind: PointerKind) -> Self {
        Self {
            id:     kind.into(),
            active: false,
            ray:    Ray {
                origin: Vec3::ZERO,
                dir:    Vec3::NEG_Z,
            },
            grasp:  0.0,
            axis:   Vec2::ZERO,
            hit:    None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputAction {
    Press,
    Release,
    Scroll(Vec2),
    Enter,
    Leave,
    MenuPress,
    MenuRelease,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputEvent {
    pub pointer: PointerId,
    pub action:  InputAction,
    pub ray:     Ray,
    pub hit:     Option<Hit>,
}
