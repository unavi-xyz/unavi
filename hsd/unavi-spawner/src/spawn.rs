//! Drops a cube on whatever the player is pointing at.

use wired_guest::math::{
    Color,
    Ray,
    Transform,
    Vec3,
};

use crate::{
    palette,
    unavi::shapes::api::Cuboid,
    wired::{
        core::ids::DocumentId,
        physics::simulation::{
            RayFilter,
            raycast,
        },
        scene::{
            document::{
                Anchor,
                Document,
                create_document,
                open_document,
            },
            properties::{
                Property,
                RigidBody,
            },
        },
    },
};

const SIZE: f32 = 0.3;
const HALF: f32 = SIZE * 0.5;

/// Furthest ahead a cube lands when the ray finds nothing.
const REACH: f32 = 6.0;
/// Nearest it can land, so pointing at a wall an arm's length away does not
/// drop one inside the player.
const MIN_DIST: f32 = 1.2;

/// Instantiates the selected shape as its own published document, on
/// whatever is being pointed at.
pub fn spawn(color: Color, cam: &Transform) -> anyhow::Result<()> {
    let doc = create_document()?;

    // `set-doc` takes the document by value; a fresh handle is spent on it
    // (rather than `doc` itself) so the mesh lands in the minted document
    // and `doc` stays usable for the edits and commit below. Same pattern as
    // `example-unavi-vui/fruit.rs`.
    let cuboid = Cuboid::new(Vec3::splat(SIZE));
    cuboid.set_doc(reopen(doc.id())?);
    let cube = cuboid.mesh()?;

    doc.local()
        .set(cube, Property::Collider(cuboid.collider()))
        .set(cube, Property::RigidBody(RigidBody::dynamic()))
        .set(cube, Property::Material(palette::cube(color)))
        .set(
            cube,
            Property::Transform(Transform {
                translation: landing(cam),
                rotation:    cam.rotation,
                scale:       Vec3::ONE,
            }),
        )
        .flush()?;

    // `local` never leaves this peer, so the cube reaches others only once
    // committed. Committing every key, `parent` included, makes the locally
    // created prims durable along with the mesh `Cuboid::mesh` wrote.
    let keys: Vec<_> = doc
        .prims()
        .into_iter()
        .flat_map(|prim| doc.keys(prim).into_iter().map(move |key| (prim, key)))
        .collect();
    doc.commit(&keys)?;

    doc.place(Anchor::Space, Transform::IDENTITY)?;

    Ok(())
}

/// Opens a fresh handle onto `id`, for an API that takes a document by
/// value while the caller keeps using its own handle afterward.
fn reopen(id: DocumentId) -> anyhow::Result<Document> {
    open_document(id)?.ok_or_else(|| anyhow::anyhow!("the minted document is no longer loaded"))
}

/// Where a cube goes: resting against whatever is in front of the player, out
/// to [`REACH`] when nothing is, and never nearer than [`MIN_DIST`].
///
/// A miss is not a failure — it means open air, which is a legitimate place to
/// put something — so it lands at arm's length rather than not at all.
fn landing(cam: &Transform) -> Vec3 {
    let dir = cam.forward();

    let at = match raycast(
        Ray {
            origin:    cam.translation,
            direction: dir,
        },
        REACH,
        &RayFilter {
            exclude_local_agent: true,
            exclude_documents:   Vec::new(),
        },
    ) {
        Ok(Some(hit)) => hit.point + hit.normal * HALF,
        Ok(None) => cam.translation + dir * REACH,
        Err(err) => {
            eprintln!("spawner: raycast failed, placing ahead instead: {err:?}");
            cam.translation + dir * REACH
        }
    };

    if (at - cam.translation).length() < MIN_DIST {
        return cam.translation + dir * MIN_DIST;
    }
    at
}
