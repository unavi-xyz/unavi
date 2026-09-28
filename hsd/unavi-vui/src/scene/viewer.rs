//! The local agent's eye, and where a surface stands relative to it.

use std::cell::RefCell;

use wired_prelude::prelude::*;

use crate::wired::{
    agent::api::local_camera,
    scene::types::Prim,
};

/// The pose every surface is aimed from.
///
/// The camera proxy appears once the local agent's avatar has loaded, so the
/// lookup retries rather than failing for the run.
pub struct Viewer(RefCell<Option<Prim>>);

impl Viewer {
    #[must_use]
    pub const fn new() -> Self {
        Self(RefCell::new(None))
    }

    #[must_use]
    pub fn pose(&self) -> Option<Transform> {
        let mut camera = self.0.borrow_mut();
        if camera.is_none() {
            *camera = local_camera().ok();
        }
        let pose = camera.as_ref()?.global_xform();
        // A proxy whose transform snapshot has not been captured yet reads as
        // identity; anchoring a surface to that would plant it in the ground.
        (pose != Transform::IDENTITY).then_some(pose)
    }
}

/// Yaw-only rotation facing `forward`, so a surface stands upright regardless
/// of where the viewer was looking when it was placed.
#[must_use]
pub fn yaw_only(forward: Vec3) -> Quat {
    let theta = (-forward.x).atan2(-forward.z);
    Quat::new(0.0, (theta * 0.5).sin(), 0.0, (theta * 0.5).cos())
}

/// The direction the viewer faces, flattened onto the ground plane.
#[must_use]
pub fn facing(eye: &Transform) -> Vec3 {
    let forward = eye.rotation * Vec3::new(0.0, 0.0, -1.0);
    Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero()
}
