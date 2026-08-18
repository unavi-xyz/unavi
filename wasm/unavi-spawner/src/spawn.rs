use wired_prelude::prelude::*;

use crate::{
    palette,
    unavi::shapes::api::Cuboid,
    wired::{
        physics::api::raycast,
        scene::{
            api::{
                create_document,
                sync_document,
            },
            types::{
                RigidBody,
                RigidBodyKind,
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
/// The ray starts clear of the player's own collider, which is otherwise the
/// first thing it hits.
const RAY_START: f32 = 0.4;

/// Instantiates the selected prefab as its own published document, on
/// whatever is being pointed at.
pub fn spawn(color: Color, cam: &Transform) -> anyhow::Result<()> {
    let doc = create_document()?;
    let id = doc.id();

    let cuboid = Cuboid::new(Vec3::splat(SIZE));
    cuboid.set_doc(doc);
    let cube = cuboid.mesh();
    cube.set_collider(Some(cuboid.collider()))?;
    cube.set_rigid_body(Some(RigidBody {
        kind:            RigidBodyKind::Dynamic,
        angular_damping: None,
        friction:        None,
        linear_damping:  None,
        mass:            None,
        restitution:     None,
    }))?;
    cube.set_material(Some(palette::cube(color)))?;

    cube.set_xform(Some(Transform {
        translation: landing(cam),
        rotation:    cam.rotation,
        scale:       Vec3::ONE,
    }))?;

    sync_document(&id)?;
    Ok(())
}

/// Where a cube goes: resting against whatever is in front of the player, out
/// to [`REACH`] when nothing is, and never nearer than [`MIN_DIST`].
///
/// A miss is not a failure — it means open air, which is a legitimate place to
/// put something — so it lands at arm's length rather than not at all.
fn landing(cam: &Transform) -> Vec3 {
    let dir = cam.forward();
    let origin = cam.translation + dir * RAY_START;

    let at = match raycast(origin, dir, REACH - RAY_START) {
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
