//! Freezing a body where it was let go, until it is thawed.
//!
//! A frozen body is kinematic at its frozen pose, both written to the shared
//! layer, so every present peer sees it hang there and it lasts the session.
//! Keeping it past the session is a build tool's commit, never the physgun's.

use wired_guest::math::Vec3;

use crate::{
    pose_in_parent,
    wired::{
        core::ids::DocumentId,
        physics::simulation::set_velocity,
        scene::{
            document::Document,
            properties::{
                Property,
                PropertyKey,
                RigidBody,
                RigidBodyKind,
            },
        },
    },
};

/// The bodies this physgun froze and has not thawed.
#[derive(Default)]
pub struct Frozen(Vec<(DocumentId, (u64, u64))>);

impl Frozen {
    /// Makes `prim` kinematic where it is now.
    pub fn freeze(&mut self, doc: &Document, prim: (u64, u64)) -> anyhow::Result<()> {
        let body = doc
            .world_transform(prim)
            .map_err(|err| anyhow::anyhow!("where the body is: {err:?}"))?;
        let local = pose_in_parent(doc, prim, body)
            .ok_or_else(|| anyhow::anyhow!("its parent or document is not placed"))?;

        // A kinematic body still moves by its velocity, so a body frozen
        // mid-throw would otherwise drift off at it.
        set_velocity(doc, prim, Some(Vec3::ZERO), Some(Vec3::ZERO))
            .map_err(|err| anyhow::anyhow!("stopping the body: {err:?}"))?;
        doc.shared()
            .set(prim, Property::RigidBody(with_kind(doc, prim, RigidBodyKind::Kinematic)))
            .set(prim, Property::Transform(local))
            .flush()?;

        let key = (doc.id(), prim);
        if !self.0.contains(&key) {
            self.0.push(key);
        }
        Ok(())
    }

    /// Lets a body this physgun froze move again, from where it hangs.
    /// Answers whether it was one.
    pub fn thaw(&mut self, doc: &Document, prim: (u64, u64)) -> anyhow::Result<bool> {
        let key = (doc.id(), prim);
        let Some(index) = self.0.iter().position(|frozen| *frozen == key) else {
            return Ok(false);
        };
        doc.shared()
            .set(prim, Property::RigidBody(with_kind(doc, prim, RigidBodyKind::Dynamic)))
            .flush()?;
        self.0.swap_remove(index);
        Ok(true)
    }
}

/// `prim`'s rigid body as it stands, with its kind replaced, so freezing
/// keeps its mass, friction and damping.
fn with_kind(doc: &Document, prim: (u64, u64), kind: RigidBodyKind) -> RigidBody {
    let mut body = match doc.get(prim, &PropertyKey::RigidBody) {
        Some(Property::RigidBody(body)) => body,
        _ => RigidBody::dynamic(),
    };
    body.kind = kind;
    body
}
