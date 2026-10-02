//! The prop a physgun has grabbed: its offset from the camera, and the
//! velocity controller that drags it toward that offset.

use wired_guest::math::{
    Quat,
    Ray,
    Transform,
    Vec3,
};

use crate::wired::{
    peer::authority::{
        release_hold,
        take_hold,
    },
    physics::simulation::{
        RayFilter,
        raycast,
        set_velocity,
    },
    scene::{
        document::{
            Document,
            open_document,
        },
        properties::{
            Collider,
            Property,
            PropertyKey,
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
const MIN_DIST: f32 = 1.0;

/// A dynamic body dragged by the physgun; grab point and orientation are
/// stored in camera-local space.
pub struct Held {
    doc:        Document,
    prim:       (u64, u64),
    offset:     Vec3,
    offset_rot: Quat,
    /// The gravity scale to restore on release; `none` when the prop is not
    /// one this script may write, so there was nothing to zero.
    gravity:    Option<f32>,
    /// Where the ray landed, in body-local space; the beam attaches here, so
    /// grabbing a corner drags that corner.
    grab_local: Vec3,
}

impl Held {
    #[must_use]
    pub fn grab(cam: &Transform) -> Option<Self> {
        let hit = match raycast(
            Ray {
                origin:    cam.translation,
                direction: cam.forward(),
            },
            RAY_MAX,
            &RayFilter {
                exclude_local_agent: true,
                exclude_documents:   Vec::new(),
            },
        ) {
            Ok(Some(hit)) => hit,
            Ok(None) => {
                println!("physgun: raycast miss");
                return None;
            }
            Err(err) => {
                eprintln!("physgun: raycast error {err:?}");
                return None;
            }
        };

        let doc = match open_document(hit.target.document) {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                println!("physgun: hit document not loaded");
                return None;
            }
            Err(err) => {
                eprintln!("physgun: open_document error {err:?}");
                return None;
            }
        };
        let prim = hit.target.prim;

        // Read before taking the hold or zeroing gravity: a failure here
        // must leave nothing to undo, rather than leaking a hold that was
        // never released and a gravity-scale write that was never restored.
        let body = match doc.world_transform(prim) {
            Ok(body) => body,
            Err(err) => {
                eprintln!("physgun: world_transform error {err:?}");
                return None;
            }
        };

        if let Err(err) = take_hold(&doc) {
            eprintln!("physgun: take_hold failed (holding anyway): {err:?}");
        }

        // Writing `gravity-scale` only succeeds on a document this script
        // may write; grabbing someone else's prop leaves gravity alone, so
        // it still falls while the controller fights to hold its position.
        let gravity = match doc.get(prim, &PropertyKey::GravityScale) {
            Some(Property::GravityScale(g)) => g,
            _ => 1.0,
        };
        let gravity = match doc.local().set(prim, Property::GravityScale(0.0)).flush() {
            Ok(()) => Some(gravity),
            Err(err) => {
                eprintln!("physgun: could not disable gravity on the held prop: {err:?}");
                None
            }
        };

        let grab_local = body.rotation.inverse() * (hit.point - body.translation);

        Some(Self {
            doc,
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
        match self.doc.get(self.prim, &PropertyKey::Collider) {
            Some(Property::Collider(collider)) => Some(collider),
            _ => None,
        }
    }

    #[must_use]
    pub fn body(&self) -> Transform {
        self.doc
            .world_transform(self.prim)
            .unwrap_or(Transform::IDENTITY)
    }

    /// The clicked point's current world position; read at render rate by the
    /// beam, as the body only steps at the fixed rate.
    #[must_use]
    pub fn grab_point(&self) -> Vec3 {
        let body = self.body();
        body.translation + body.rotation * self.grab_local
    }

    pub fn update(&self, cam: &Transform) {
        let body = self.body();
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

        if let Err(err) = set_velocity(&self.doc, self.prim, Some(vel), Some(ang_vel)) {
            eprintln!("physgun: set_velocity: {err:?}");
        }
    }

    pub fn nudge_distance(&mut self, delta: f32) {
        self.offset.z = (self.offset.z - delta).clamp(-RAY_MAX, -MIN_DIST);
    }

    /// Restores gravity and releases, keeping the body's velocity so a fast
    /// sweep throws it.
    pub fn release(&self) {
        if let Some(gravity) = self.gravity
            && let Err(err) = self
                .doc
                .local()
                .set(self.prim, Property::GravityScale(gravity))
                .flush()
        {
            eprintln!("physgun: could not restore gravity on the held prop: {err:?}");
        }
        if let Err(err) = release_hold(&self.doc, None) {
            eprintln!("physgun: release_hold: {err:?}");
        }
    }
}
