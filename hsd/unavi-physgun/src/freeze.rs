//! Freezing a body where it was let go, and thawing it again.
//!
//! A frozen body is a kinematic one, at its frozen pose. Both go through an
//! undoable edit, so where the user may write the body's document (or the
//! space referencing it) the freeze is kept, and elsewhere it lasts the
//! session. Freezing is how a thrown thing becomes a placed one.

use wired_guest::math::Vec3;

use crate::{
    pose_in_parent,
    unavi::author::api::Edit,
    wired::{
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

/// Whether `prim` is a kinematic body, which is what a frozen one is.
#[must_use]
pub fn is_frozen(doc: &Document, prim: (u64, u64)) -> bool {
    matches!(
        doc.get(prim, &PropertyKey::RigidBody),
        Some(Property::RigidBody(RigidBody {
            kind: RigidBodyKind::Kinematic,
            ..
        }))
    )
}

/// Makes `prim` kinematic where it is now.
pub fn freeze(doc: &Document, prim: (u64, u64)) -> anyhow::Result<()> {
    let body = doc
        .world_transform(prim)
        .map_err(|err| anyhow::anyhow!("where the body is: {err:?}"))?;
    let local = pose_in_parent(doc, prim, body)
        .ok_or_else(|| anyhow::anyhow!("its parent or document is not placed"))?;

    let edit = Edit::new("Freeze")?;
    edit.track(
        doc,
        &[
            (prim, PropertyKey::RigidBody),
            (prim, PropertyKey::Transform),
        ],
    );
    // A kinematic body still moves by its velocity, so a body frozen
    // mid-throw would otherwise drift off at it.
    set_velocity(doc, prim, Some(Vec3::ZERO), Some(Vec3::ZERO))
        .map_err(|err| anyhow::anyhow!("stopping the body: {err:?}"))?;
    doc.shared()
        .set(
            prim,
            Property::RigidBody(with_kind(doc, prim, RigidBodyKind::Kinematic)),
        )
        .set(prim, Property::Transform(local))
        .flush()?;
    keep(&edit);
    Ok(())
}

/// Lets a frozen body move again, from where it hangs.
pub fn thaw(doc: &Document, prim: (u64, u64)) -> anyhow::Result<()> {
    let edit = Edit::new("Unfreeze")?;
    edit.track(doc, &[(prim, PropertyKey::RigidBody)]);
    doc.shared()
        .set(
            prim,
            Property::RigidBody(with_kind(doc, prim, RigidBodyKind::Dynamic)),
        )
        .flush()?;
    keep(&edit);
    Ok(())
}

/// Commits `edit`, which only a document the user may write takes. Anywhere
/// else the shared write stands for the session.
fn keep(edit: &Edit) {
    if let Err(err) = edit.commit() {
        println!("physgun: held for this session only: {err:?}");
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
