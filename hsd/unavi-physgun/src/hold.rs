//! The prop a physgun has grabbed: its offset from the aim, and the velocity
//! controller that drags it toward that offset.

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
/// How far from vertical, as the sine of the angle, the aim must be for its
/// own forward to give the heading.
const LEVEL_ENOUGH: f32 = 0.25;

/// What the aim ray landed on.
pub struct Target {
    pub doc:   Document,
    pub prim:  (u64, u64),
    pub point: Vec3,
}

impl Target {
    /// The prim the aim ray meets first, when it belongs to a loaded document.
    #[must_use]
    pub fn under(aim: &Transform) -> Option<Self> {
        let hit = match raycast(
            Ray {
                origin:    aim.translation,
                direction: aim.forward(),
            },
            RAY_MAX,
            &RayFilter {
                exclude_local_agent: true,
                exclude_documents:   Vec::new(),
            },
        ) {
            Ok(Some(hit)) => hit,
            Ok(None) => return None,
            Err(err) => {
                eprintln!("physgun: raycast error {err:?}");
                return None;
            }
        };

        match open_document(hit.target.document) {
            Ok(Some(doc)) => Some(Self {
                doc,
                prim: hit.target.prim,
                point: hit.point,
            }),
            Ok(None) => {
                println!("physgun: hit document not loaded");
                None
            }
            Err(err) => {
                eprintln!("physgun: open_document error {err:?}");
                None
            }
        }
    }
}

/// The rotation a held body keeps its orientation against: the whole aim
/// while rotating with the wrist, else only its heading, so tilting or
/// rolling the hand leaves the body as it was.
fn reference(aim: &Transform, wrist: bool) -> Quat {
    if wrist {
        return aim.rotation;
    }
    let forward = aim.rotation * Vec3::NEG_Z;
    // Near vertical the forward's heading is noise, so the right vector, which
    // a hand pointing up or down still holds level, gives it instead.
    let yaw = if forward.x.hypot(forward.z) > LEVEL_ENOUGH {
        (-forward.x).atan2(-forward.z)
    } else {
        let right = aim.rotation * Vec3::X;
        (-right.z).atan2(right.x)
    };
    Quat::from_rotation_y(yaw)
}

/// A dynamic body dragged by the physgun. The grab point is stored in the
/// aim's local space and the orientation against [`reference`].
pub struct Held {
    doc:        Document,
    prim:       (u64, u64),
    offset:     Vec3,
    offset_rot: Quat,
    /// Whether the body turns with the whole aim rather than its heading.
    wrist:      bool,
    /// The gravity scale to restore on release; `none` when the prop is not
    /// one this script may write, so there was nothing to zero.
    gravity:    Option<f32>,
    /// Where the ray landed, in body-local space; the beam attaches here, so
    /// grabbing a corner drags that corner.
    grab_local: Vec3,
}

impl Held {
    #[must_use]
    pub fn grab(aim: &Transform, target: Target) -> Option<Self> {
        let Target { doc, prim, point } = target;

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

        let grab_local = body.rotation.inverse() * (point - body.translation);

        Some(Self {
            doc,
            prim,
            offset: aim.rotation.inverse() * (point - aim.translation),
            offset_rot: reference(aim, false).inverse() * body.rotation,
            wrist: false,
            gravity,
            grab_local,
        })
    }

    #[must_use]
    pub const fn doc(&self) -> &Document {
        &self.doc
    }

    #[must_use]
    pub const fn prim(&self) -> (u64, u64) {
        self.prim
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

    /// Switches between turning with the wrist and keeping only the heading,
    /// measured from where the body is now so it does not jump.
    pub fn set_wrist(&mut self, aim: &Transform, wrist: bool) {
        if self.wrist == wrist {
            return;
        }
        self.wrist = wrist;
        self.offset_rot = reference(aim, wrist).inverse() * self.body().rotation;
    }

    /// Turns the held body `radians` about the world's up axis.
    pub fn spin(&mut self, aim: &Transform, radians: f32) {
        let frame = reference(aim, self.wrist);
        self.offset_rot =
            frame.inverse() * Quat::from_rotation_y(radians) * frame * self.offset_rot;
    }

    pub fn nudge_distance(&mut self, delta: f32) {
        self.offset.z = (self.offset.z - delta).clamp(-RAY_MAX, -MIN_DIST);
    }

    pub fn update(&self, aim: &Transform) {
        let body = self.body();
        let target = aim.transform_point(self.offset);

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

        let target_rotation = reference(aim, self.wrist) * self.offset_rot;
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
        if let Err(err) = release_hold(&self.doc) {
            eprintln!("physgun: release_hold: {err:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aim(yaw: f32, pitch: f32, roll: f32) -> Transform {
        Transform::new(
            Vec3::ZERO,
            Quat::from(glam::Quat::from_euler(
                glam::EulerRot::YXZ,
                yaw,
                pitch,
                roll,
            )),
            Vec3::ONE,
        )
    }

    fn same(a: Quat, b: Quat) -> bool {
        glam::Quat::from(a).dot(glam::Quat::from(b)).abs() > 0.9999
    }

    #[test]
    fn the_heading_ignores_pitch_and_roll() {
        assert!(same(
            reference(&aim(0.7, 0.0, 0.0), false),
            reference(&aim(0.7, -0.5, 0.9), false)
        ));
    }

    #[test]
    fn a_hand_pointing_straight_down_keeps_its_heading() {
        let down = aim(0.7, -std::f32::consts::FRAC_PI_2, 0.0);
        assert!(same(reference(&down, false), Quat::from_rotation_y(0.7)));
    }
}
