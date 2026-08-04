use wired_prelude::prelude::*;

use crate::{
    palette,
    unavi::shapes::api::Cuboid,
    wired::scene::{
        api::{
            create_document,
            sync_document,
        },
        types::{
            RigidBody,
            RigidBodyKind,
            Xform,
        },
    },
};

const SIZE: f32 = 0.3;
const DIST: f32 = 1.5;

/// Instantiates the selected prefab (a dynamic cube for now) as its own
/// published document ahead of the camera. A future prefab picker can swap the
/// cube build for `load_hsd` behind this same entry point.
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

    let forward = cam.rotation * Vec3::new(0.0, 0.0, -1.0);
    cube.set_xform(Some(Xform {
        translation: cam.translation + forward * DIST,
        rotation:    cam.rotation,
        scale:       Vec3::ONE,
    }))?;

    sync_document(&id)?;
    Ok(())
}
