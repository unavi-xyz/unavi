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
        types::{
            Collider,
            Prim,
        },
    },
};

const RAY_MAX: f32 = 100.0;
/// Gap fraction closed per second, kept low so actuation latency cannot make
/// the tracking oscillate; the lag it leaves is what bends the beam on sweeps.
const FOLLOW: f32 = 5.5;
const MAX_SPEED: f32 = 30.0;
const SETTLE: f32 = 0.01;
const ROTATE_SETTLE: f32 = 0.01;
const RAY_START: f32 = 0.4;
const MIN_DIST: f32 = 1.0;

/// A dynamic body dragged by the physgun; grab point and orientation are
/// stored in camera-local space.
pub struct Held {
    doc:        Vec<u8>,
    prim:       Prim,
    offset:     Vec3,
    offset_rot: Quat,
    gravity:    f32,
    /// Where the ray landed, in body-local space; the beam attaches here, so
    /// grabbing a corner drags that corner.
    grab_local: Vec3,
}

impl Held {
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
        if let Err(err) = prim.set_gravity_scale(0.0) {
            // Swallowing this leaves the prop falling while the controller
            // fights to lift it, which looks like a tuning problem rather
            // than a failed write.
            eprintln!("physgun: could not disable gravity on the held prop: {err:?}");
        }
        let body = prim.global_xform();
        let grab_local = body.rotation.inverse() * (hit.point - body.translation);
        Some(Self {
            doc: hit.document,
            prim,
            offset: cam.rotation.inverse() * (hit.point - cam.translation),
            offset_rot: cam.rotation.inverse() * body.rotation,
            gravity,
            grab_local,
        })
    }

    /// The prop's collider, for building a highlight shell around it.
    #[must_use]
    pub fn collider(&self) -> Option<Collider> {
        self.prim.collider()
    }

    #[must_use]
    pub fn body(&self) -> Transform {
        self.prim.global_xform()
    }

    /// The clicked point's current world position; read at render rate by the
    /// beam, as the body only steps at the fixed rate.
    #[must_use]
    pub fn grab_point(&self) -> Vec3 {
        let body = self.prim.global_xform();
        body.translation + body.rotation * self.grab_local
    }

    pub fn update(&self, cam: &Transform) {
        let body = self.prim.global_xform();
        let target = cam.transform_point(self.offset);

        // Aim the body's centre at the grab point's target: measuring error
        // at the grab point couples the linear and angular controllers, and
        // the two fighting reads as the prop bobbing.
        let desired_centre = target - body.rotation * self.grab_local;
        let error = desired_centre - body.translation;
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
    }

    pub fn nudge_distance(&mut self, delta: f32) {
        self.offset.z = (self.offset.z - delta).clamp(-RAY_MAX, -MIN_DIST);
    }

    /// Restores gravity and releases, keeping the body's velocity so a fast
    /// sweep throws it.
    pub fn release(&self) {
        self.prim.set_gravity_scale(self.gravity).ok();
        release_authority(&self.doc).ok();
    }
}
