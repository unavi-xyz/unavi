use wired_prelude::prelude::*;

use crate::wired::{
    physics::api::{
        claim_authority,
        get_linear_velocity,
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
/// Where the body "wants" to be: desired velocity = error * STIFFNESS.
const STIFFNESS: f32 = 14.0;
const MAX_SPEED: f32 = 40.0;
/// Fraction the body's own velocity eases toward the goal each tick. Near 1 it
/// tracks the goal tightly (fast convergence, minimal overshoot) while still
/// carrying enough velocity to throw on release.
const RESPONSIVENESS: f32 = 0.8;
/// Within this distance the body is considered settled and commanded to stop,
/// so it holds still instead of jittering on the target.
const SETTLE: f32 = 0.02;
/// Start the ray ahead of the camera so it clears the player's own collider.
const RAY_START: f32 = 0.4;
const MIN_DIST: f32 = 1.0;

/// A dynamic body grabbed by the physgun, held at a fixed distance ahead of the
/// aim while the caller owns physics authority over its document.
pub struct Held {
    doc:  Vec<u8>,
    prim: Prim,
    dist: f32,
}

impl Held {
    /// Raycasts from the camera; on a hit, claims authority over the hit
    /// document and returns the grab handle.
    pub fn grab(cam: &Transform) -> Option<Self> {
        let dir = cam.rotation * Vec3::new(0.0, 0.0, -1.0);
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
        println!("physgun: hit dist={} prim={}", hit.distance, hit.prim);

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

        // Best-effort: locally-owned bodies are already authoritative, and a
        // failed claim should not block holding.
        if let Err(err) = claim_authority(&hit.document) {
            println!("physgun: claim_authority failed (holding anyway): {err:?}");
        }
        println!("physgun: grabbed");
        Some(Self {
            doc: hit.document,
            prim,
            dist: RAY_START + hit.distance,
        })
    }

    /// Drives the body toward the aim point and returns its current world
    /// position for the laser to track.
    pub fn update(&self, cam: &Transform) -> Vec3 {
        let forward = cam.rotation * Vec3::new(0.0, 0.0, -1.0);
        let target = cam.translation + forward * self.dist;
        let current = self.prim.global_xform().translation;

        let error = target - current;
        let mut desired = if error.length() < SETTLE {
            Vec3::ZERO
        } else {
            error * STIFFNESS
        };
        if desired.length() > MAX_SPEED {
            desired = desired.normalize() * MAX_SPEED;
        }
        let cur_v = get_linear_velocity(&self.prim).unwrap_or(Vec3::ZERO);
        set_linear_velocity(&self.prim, cur_v.lerp(desired, RESPONSIVENESS)).ok();
        set_angular_velocity(&self.prim, Vec3::ZERO).ok();
        current
    }

    /// Adjusts the hold distance (physgun scroll / push-pull).
    pub fn nudge_distance(&mut self, delta: f32) {
        self.dist = (self.dist + delta).clamp(MIN_DIST, RAY_MAX);
    }

    /// Releases without zeroing velocity, so a body flung by a fast aim sweep
    /// keeps its momentum and is thrown.
    pub fn release(&self) {
        release_authority(&self.doc).ok();
    }
}
