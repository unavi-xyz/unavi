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
const RAY_START: f32 = 0.4;
const MIN_DIST: f32 = 1.0;

/// A dynamic body dragged by the physgun. The grab point is stored in
/// camera-local space, so the body holds its position until the camera moves.
pub struct Held {
    doc:     Vec<u8>,
    prim:    Prim,
    offset:  Vec3,
    gravity: f32,
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
        let body = prim.global_xform().translation;
        Some(Self {
            doc: hit.document,
            prim,
            offset: cam.rotation.inverse() * (body - cam.translation),
            gravity,
        })
    }

    /// Drags the body toward the grab point and returns its current world
    /// position for the laser to track.
    pub fn update(&self, cam: &Transform) -> Vec3 {
        let target = cam.transform_point(self.offset);
        let current = self.prim.global_xform().translation;

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
        set_angular_velocity(&self.prim, Vec3::ZERO).ok();
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
