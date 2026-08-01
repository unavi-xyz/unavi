use wired_prelude::prelude::*;

use crate::wired::{
    physics::api::{
        claim_authority,
        raycast,
        release_authority,
        set_angular_velocity,
        set_linear_velocity,
    },
    scene::{
        api::get_document,
        types::Prim,
    },
};

const RAY_MAX: f32 = 100.0;
/// Gap fraction closed per second; kept low so actuation latency can't make the
/// tracking oscillate. The lag it leaves is the inertia.
const FOLLOW: f32 = 10.0;
const MAX_SPEED: f32 = 30.0;
const SETTLE: f32 = 0.01;
const ROTATE_SETTLE: f32 = 0.01;
const RAY_START: f32 = 0.4;
const MIN_DIST: f32 = 1.0;

/// A dynamic body dragged by the physgun. The grab point and orientation are
/// stored in camera-local space, so the body holds its pose until the camera
/// moves, and turns as the camera looks around.
pub struct Held {
    doc:        Vec<u8>,
    prim:       Prim,
    offset:     Vec3,
    offset_rot: Quat,
    gravity:    f32,
}

impl Held {
    /// Raycasts from the camera; on a hit, claims authority, disables the
    /// body's gravity, and returns the grab handle.
    pub fn grab(cam: &Transform) -> Option<Self> {
        let dir = cam.forward();
        let origin = cam.translation + dir * RAY_START;
        let hit = match raycast(origin, dir, RAY_MAX) {
            Ok(Some(hit)) => hit,
            Ok(None) => {
                println!("physgun: raycast miss");
                return None;
            }
            Err(err) => {
                println!("physgun: raycast error {err:?}");
                return None;
            }
        };

        let document = match get_document(&hit.document) {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                println!("physgun: hit document not found");
                return None;
            }
            Err(err) => {
                println!("physgun: get_document error {err:?}");
                return None;
            }
        };
        let Some(prim) = document.get_prim(&hit.prim) else {
            println!("physgun: prim {} not present in document", hit.prim);
            return None;
        };

        if let Err(err) = claim_authority(&hit.document) {
            println!("physgun: claim_authority failed (holding anyway): {err:?}");
        }

        let gravity = prim.gravity_scale();
        prim.set_gravity_scale(0.0).ok();
        let body = prim.global_xform();
        Some(Self {
            doc: hit.document,
            prim,
            offset: cam.rotation.inverse() * (body.translation - cam.translation),
            offset_rot: cam.rotation.inverse() * body.rotation,
            gravity,
        })
    }

    /// Drags the body toward the grab point and turns it to match the
    /// camera's look direction, returning its current world position for the
    /// laser to track.
    pub fn update(&self, cam: &Transform) -> Vec3 {
        let body = self.prim.global_xform();
        let current = body.translation;

        let target = cam.transform_point(self.offset);
        let error = target - current;
        let mut vel = if error.length() < SETTLE {
            Vec3::ZERO
        } else {
            error * FOLLOW
        };
        let speed = vel.length();
        if speed > MAX_SPEED {
            vel *= MAX_SPEED / speed;
        }
        set_linear_velocity(&self.prim, vel).ok();

        let target_rotation = cam.rotation * self.offset_rot;
        let mut rotation_diff = target_rotation * body.rotation.inverse();
        // Ensure shortest path (quaternion double-cover: q and -q are the same
        // rotation)
        if rotation_diff.w < 0.0 {
            rotation_diff = -rotation_diff;
        }
        let (axis, angle) = rotation_diff.normalize().to_axis_angle();
        let ang_vel = if angle.abs() < ROTATE_SETTLE {
            Vec3::ZERO
        } else {
            axis * angle * FOLLOW
        };
        set_angular_velocity(&self.prim, ang_vel).ok();

        current
    }

    /// Adjusts the hold distance along the view (physgun scroll / push-pull).
    pub fn nudge_distance(&mut self, delta: f32) {
        self.offset.z = (self.offset.z - delta).clamp(-RAY_MAX, -MIN_DIST);
    }

    /// Restores gravity and releases without zeroing velocity, so a body flung
    /// by a fast aim sweep keeps its momentum and is thrown.
    pub fn release(&self) {
        self.prim.set_gravity_scale(self.gravity).ok();
        release_authority(&self.doc).ok();
    }
}
