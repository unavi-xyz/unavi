use bevy::prelude::*;
use unavi_input::pointer::{
    PointerHit,
    PointerKind,
};

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
    pub kind:    PointerKind,
    pub active:  bool,
    pub ray:     Ray,
    pub trigger: f32,
    pub grip:    f32,
    pub axis:    Vec2,
    pub hit:     Option<Hit>,
}

impl Pointer {
    /// A pointer the rig never spawned: no hands on desktop, no screen
    /// pointer in VR. Still listed, so a script can see it is not there.
    #[must_use]
    pub const fn inactive(kind: PointerKind) -> Self {
        Self {
            kind,
            active: false,
            ray: Ray {
                origin: Vec3::ZERO,
                dir:    Vec3::NEG_Z,
            },
            trigger: 0.0,
            grip: 0.0,
            axis: Vec2::ZERO,
            hit: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputAction {
    Press,
    Release,
    GripPress,
    GripRelease,
    Scroll(Vec2),
    Enter,
    Leave,
    MenuPress,
    MenuRelease,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputEvent {
    pub pointer: PointerKind,
    pub action:  InputAction,
    pub ray:     Ray,
    pub hit:     Option<Hit>,
}
