//! The edits an inspection makes: nudging the transform from its rows,
//! locking, making a reference local and deleting. Each is undoable.

use wired_guest::math::{
    Quat,
    Transform,
    Vec3,
};

use super::{
    Inspection,
    row,
};
use crate::{
    is_locked,
    pose_in_parent,
    unavi::author::api::Edit,
    wired::{
        peer::authority::{
            is_holder,
            release_hold,
            take_hold,
        },
        physics::simulation::set_velocity,
        scene::{
            document::{
                Layer,
                copy_document,
            },
            properties::{
                Property,
                PropertyKey,
                RigidBody,
                RigidBodyKind,
            },
        },
    },
};

/// The smallest uniform scale a nudge brings an element down to.
const MIN_SCALE: f32 = 0.01;

/// A change a scrubbed transform row asks for, by how much it was dragged.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Nudge {
    /// Along the parent's X, Y or Z, in metres.
    Move(usize, f32),
    /// About the parent's vertical, in degrees.
    Turn(f32),
    /// To the uniform scale, by this much.
    Grow(f32),
}

impl Nudge {
    /// The nudge a scrub of `by` on part `part` of row `id` asks for, if it
    /// is a transform row.
    #[must_use]
    pub const fn of(id: u32, part: u32, by: f32) -> Option<Self> {
        match id {
            row::POSITION if part < 3 => Some(Self::Move(part as usize, by)),
            row::TURN => Some(Self::Turn(by)),
            row::SCALE => Some(Self::Grow(by)),
            _ => None,
        }
    }

    /// `pose` with this nudge applied.
    #[must_use]
    pub fn apply(self, pose: Transform) -> Transform {
        match self {
            Self::Move(axis, by) => {
                let mut translation = pose.translation;
                translation[axis] += by;
                Transform {
                    translation,
                    ..pose
                }
            }
            Self::Turn(degrees) => Transform {
                rotation: Quat::from_rotation_y(degrees.to_radians()) * pose.rotation,
                ..pose
            },
            // Scaled as a whole, keeping the proportions a stretched element
            // has.
            Self::Grow(by) => {
                let now = uniform_scale(pose.scale);
                let next = (now + by).max(MIN_SCALE);
                Transform {
                    scale: pose.scale * (next / now.max(f32::EPSILON)),
                    ..pose
                }
            }
        }
    }
}

/// The scale a row shows for `scale`: its largest axis.
#[must_use]
pub fn uniform_scale(scale: Vec3) -> f32 {
    scale.abs().max_element()
}

/// A scrub in progress: the transform edit it has open, and the pose it has
/// set so far.
pub struct Held {
    edit:   Edit,
    pose:   Transform,
    /// The dynamic body the scrub holds still, to hand back to physics on
    /// release.
    paused: Option<Paused>,
}

/// A dynamic body held still for a scrub.
struct Paused {
    /// The body as it was.
    body:  RigidBody,
    /// The layer the pause landed in, which the release must write too: a
    /// release in a weaker layer would leave the body still.
    layer: Layer,
    /// Whether the scrub took the hold, so the release gives up only a hold
    /// it took, not one another tool has.
    took:  bool,
}

impl Inspection {
    /// The prim's transform as the rows show it: where a dynamic body is
    /// now, since physics carries it from its stated pose, else as stated.
    #[must_use]
    pub fn pose(&self) -> Option<Transform> {
        let Some(Property::Transform(stated)) =
            self.hit_doc.get(self.hit_prim, &PropertyKey::Transform)
        else {
            return None;
        };
        if self.dynamic_body().is_none() {
            return Some(stated);
        }
        let live = self
            .hit_doc
            .world_transform(self.hit_prim)
            .ok()
            .and_then(|world| pose_in_parent(&self.hit_doc, self.hit_prim, world));
        Some(live.map_or(stated, |live| Transform {
            scale: stated.scale,
            ..live
        }))
    }

    /// The pose a scrub in progress has set.
    #[must_use]
    pub(super) fn held_pose(&self) -> Option<Transform> {
        self.held.as_ref().map(|held| held.pose)
    }

    /// Applies `nudge`, opening the transform edit on the first of a scrub.
    pub fn nudge(&mut self, nudge: Nudge) -> anyhow::Result<()> {
        if self.held.is_none() {
            if !self.editable() {
                return Ok(());
            }
            let Some(pose) = self.pose() else {
                return Ok(());
            };
            let edit = Edit::new("Edit transform")?;
            edit.track(&self.hit_doc, &[(self.hit_prim, PropertyKey::Transform)]);
            let paused = self.dynamic_body().and_then(|body| self.pause(body, pose));
            self.held = Some(Held { edit, pose, paused });
        }
        let Some(held) = &mut self.held else {
            return Ok(());
        };
        held.pose = nudge.apply(held.pose);
        let pose = held.pose;
        self.write(&[Property::Transform(pose)])?;
        Ok(())
    }

    fn dynamic_body(&self) -> Option<RigidBody> {
        match self.hit_doc.get(self.hit_prim, &PropertyKey::RigidBody) {
            Some(Property::RigidBody(body)) if body.kind == RigidBodyKind::Dynamic => Some(body),
            _ => None,
        }
    }

    /// Holds `body` still where it is, so physics neither fights the scrub's
    /// writes nor lets it fall between them, and so this peer drives its
    /// motion for everyone.
    fn pause(&self, body: RigidBody, local: Transform) -> Option<Paused> {
        let took = !is_holder(&self.hit_doc).unwrap_or(false);
        if took && let Err(err) = take_hold(&self.hit_doc) {
            eprintln!("inspector: take hold: {err:?}");
        }
        self.stop();
        // Stated where it is in the same batch, so turning kinematic cannot
        // snap it back to its stated pose.
        let still = RigidBody {
            kind: RigidBodyKind::Kinematic,
            ..body
        };
        match self.write(&[Property::RigidBody(still), Property::Transform(local)]) {
            Ok(layer) => Some(Paused { body, layer, took }),
            Err(err) => {
                eprintln!("inspector: holding the body still: {err:?}");
                self.release(took);
                None
            }
        }
    }

    /// Hands a paused body back to physics, at rest where the scrub left it.
    fn resume(&self, paused: &Paused) {
        let restored = self
            .batch(paused.layer)
            .set(self.hit_prim, Property::RigidBody(paused.body))
            .flush();
        if let Err(err) = restored {
            eprintln!("inspector: letting the body go: {err:?}");
        }
        self.stop();
        self.release(paused.took);
    }

    fn release(&self, took: bool) {
        if took && let Err(err) = release_hold(&self.hit_doc) {
            eprintln!("inspector: release hold: {err:?}");
        }
    }

    fn stop(&self) {
        if let Err(err) = set_velocity(
            &self.hit_doc,
            self.hit_prim,
            Some(Vec3::ZERO),
            Some(Vec3::ZERO),
        ) {
            eprintln!("inspector: stopping the body: {err:?}");
        }
    }

    /// Writes `sets` for everyone, or for this peer only when the document
    /// refuses a shared write. Returns the layer they landed in.
    fn write(&self, sets: &[Property]) -> anyhow::Result<Layer> {
        let apply = |layer: Layer| {
            let mut batch = self.batch(layer);
            for set in sets {
                batch = batch.set(self.hit_prim, set.clone());
            }
            batch.flush()
        };
        if let Err(err) = apply(Layer::Shared) {
            eprintln!("inspector: editing for this peer only: {err:?}");
            apply(Layer::Local)?;
            return Ok(Layer::Local);
        }
        Ok(Layer::Shared)
    }

    fn batch(&self, layer: Layer) -> crate::Batch<'_> {
        match layer {
            Layer::Shared => self.hit_doc.shared(),
            Layer::Local => self.hit_doc.local(),
        }
    }

    /// Keeps the transform edit a scrub left open, so ending an inspection
    /// mid-scrub still files it, and lets a body it held still move again.
    pub fn finish(&mut self) {
        let Some(held) = self.held.take() else {
            return;
        };
        if let Some(paused) = held.paused {
            self.resume(&paused);
        }
        if let Err(err) = held.edit.commit() {
            eprintln!("inspector: keep: {err:?}");
        }
    }

    /// Removes this prim and everything beneath it, undoably, deepest first
    /// so no descendant is left stated in the document.
    pub(super) fn delete(&self) -> anyhow::Result<()> {
        if self.locked() {
            anyhow::bail!("it is locked");
        }
        let edit = Edit::new("Delete")?;
        edit.track_removal(&self.hit_doc, self.hit_prim)?;
        let mut prims = vec![self.hit_prim];
        let mut next = 0;
        while let Some(&prim) = prims.get(next) {
            prims.extend(self.hit_doc.children(prim));
            next += 1;
        }
        let mut batch = self.hit_doc.local();
        for prim in prims.into_iter().rev() {
            batch = batch.remove(prim);
        }
        batch.flush()?;
        edit.commit()?;
        Ok(())
    }

    /// Whether this prim is locked, here or above it, so editing tools leave
    /// it be.
    pub(super) fn locked(&self) -> bool {
        is_locked(&self.at(self.hit_prim))
    }

    /// Whether this prim states a lock itself, rather than taking one from
    /// above.
    pub(super) fn locked_here(&self) -> bool {
        matches!(
            self.hit_doc.get(
                self.hit_prim,
                &PropertyKey::Custom(wired_guest::LOCKED.to_owned()),
            ),
            Some(Property::Custom((_, value))) if value.first() == Some(&1)
        )
    }

    /// Locks or unlocks this prim and everything beneath it, undoably.
    pub(super) fn lock(&self, on: bool) -> anyhow::Result<()> {
        let key = PropertyKey::Custom(wired_guest::LOCKED.to_owned());
        let edit = Edit::new(if on { "Lock" } else { "Unlock" })?;
        edit.track(&self.hit_doc, &[(self.hit_prim, key.clone())]);
        let batch = self.hit_doc.shared();
        let batch = if on {
            batch.set(
                self.hit_prim,
                Property::Custom((wired_guest::LOCKED.to_owned(), vec![1])),
            )
        } else {
            batch.clear(self.hit_prim, key)
        };
        batch.flush()?;
        edit.commit()?;
        Ok(())
    }

    /// Replaces the reference with a copy of what it shows, undoably, so the
    /// object here stops following its source. Overrides kept on the site
    /// still apply, since they are keyed by the site.
    pub(super) fn make_local(&self) -> anyhow::Result<()> {
        let site = self
            .site
            .ok_or_else(|| anyhow::anyhow!("no reference places this"))?;
        let copy = copy_document(&self.hit_doc)?;
        let edit = Edit::new("Make local")?;
        edit.track(&self.placed_in, &[(site, PropertyKey::Reference)]);
        self.placed_in
            .local()
            .set(site, Property::Reference(copy.id()))
            .flush()?;
        edit.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn yaw(pose: Transform) -> f32 {
        glam::Quat::from(pose.rotation)
            .to_euler(glam::EulerRot::YXZ)
            .0
            .to_degrees()
    }

    #[test]
    fn each_position_row_moves_along_its_own_axis() {
        assert_eq!(Nudge::of(row::POSITION, 1, 0.5), Some(Nudge::Move(1, 0.5)));
        assert_eq!(Nudge::of(row::POSITION, 3, 0.5), None);
        let moved = Nudge::Move(1, 0.5).apply(Transform::IDENTITY);
        assert_eq!(moved.translation, Vec3::new(0.0, 0.5, 0.0));
    }

    #[test]
    fn a_turn_adds_to_the_yaw_the_row_shows() {
        let tilted = Transform {
            rotation: Quat::from(glam::Quat::from_euler(glam::EulerRot::YXZ, 0.3, 0.2, 0.0)),
            ..Transform::IDENTITY
        };
        let turned = Nudge::Turn(30.0).apply(tilted);
        assert!((yaw(turned) - yaw(tilted) - 30.0).abs() < 1.0e-3);
    }

    #[test]
    fn growing_keeps_proportions_and_never_reaches_zero() {
        let stretched = Transform {
            scale: Vec3::new(2.0, 1.0, 1.0),
            ..Transform::IDENTITY
        };
        let grown = Nudge::Grow(2.0).apply(stretched);
        assert!((grown.scale - Vec3::new(4.0, 2.0, 2.0)).length() < 1.0e-5);
        let shrunk = Nudge::Grow(-10.0).apply(stretched);
        assert!((uniform_scale(shrunk.scale) - MIN_SCALE).abs() < 1.0e-6);
    }
}
