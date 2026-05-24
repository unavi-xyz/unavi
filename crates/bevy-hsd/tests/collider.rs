use std::time::Duration;

use avian3d::prelude::Collider;
use bevy::prelude::*;
use bevy_hsd::attributes::collider::{DisabledCollider, HsdCollider};
use bytemuck::cast_slice;
use hsd::{
    HSD_CONTAINER_ID, PrimMeta,
    attributes::{Attribute, Attributes, attributes_map, collider::ColliderAttr},
};
use loro_surgeon::{Reconcile, bytes::ByteArray, reconcile::RootReconciler};
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

#[traced_test]
#[rstest]
fn test_collider_lifecycle(mut ctx: TestContext) {
    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
    let root = tree.create(None).expect("create");
    let meta = tree.get_meta(root).expect("get meta");

    reconcile_collider(&meta, ColliderAttr::Sphere(0.5));

    ctx.doc.commit();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut q = world.query::<(&HsdCollider, &Collider)>();
    assert!(
        q.iter(world).next().is_some(),
        "HsdCollider + Collider expected"
    );

    let attrs = attributes_map(&meta).expect("attributes map");
    attrs.delete(ColliderAttr::KEY).expect("delete");

    ctx.doc.commit();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut q_marker = world.query::<&HsdCollider>();
    assert!(
        q_marker.iter(world).next().is_none(),
        "HsdCollider should be removed"
    );
    let mut q_col = world.query::<&Collider>();
    assert!(
        q_col.iter(world).next().is_none(),
        "Collider should be removed"
    );
}

#[traced_test]
#[rstest]
fn test_collider_invalid_sphere(mut ctx: TestContext) {
    for bad_r in [0.0_f64, -1.0, f64::NAN, f64::INFINITY] {
        let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
        let root = tree.create(None).expect("create");
        let meta = tree.get_meta(root).expect("get meta");

        reconcile_collider(&meta, ColliderAttr::Sphere(bad_r));

        ctx.doc.commit();
        ctx.app.update();

        let world = ctx.app.world_mut();
        let mut q_marker = world.query::<&HsdCollider>();
        assert!(
            q_marker.iter(world).next().is_some(),
            "HsdCollider marker expected even for invalid"
        );
        let mut q_col = world.query::<&Collider>();
        assert!(
            q_col.iter(world).next().is_none(),
            "Collider should NOT be inserted for invalid sphere r={bad_r}"
        );

        assert!(logs_contain("radius must be positive"));
    }
}

#[traced_test]
#[rstest]
fn test_collider_invalid_cuboid(mut ctx: TestContext) {
    for (x, y, z) in [(0.0_f64, 1.0, 1.0), (1.0, -1.0, 1.0), (1.0, 1.0, f64::NAN)] {
        let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
        let root = tree.create(None).expect("create");
        let meta = tree.get_meta(root).expect("get meta");

        reconcile_collider(&meta, ColliderAttr::Cuboid { x, y, z });

        ctx.doc.commit();
        ctx.app.update();

        let world = ctx.app.world_mut();
        let mut q_col = world.query::<&Collider>();
        assert!(
            q_col.iter(world).next().is_none(),
            "no Collider for invalid cuboid ({x},{y},{z})"
        );

        assert!(logs_contain("all dimensions must be positive"));
    }
}

#[traced_test]
#[rstest]
fn test_collider_trimesh_blob(#[from(ctx_wds)] mut ctx: TestContext) {
    const VERTS: [[f32; 3]; 4] = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ];
    const IDXS: [[u32; 3]; 4] = [[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]];

    let vertex_hash = ctx.upload_blob(cast_slice::<[f32; 3], u8>(&VERTS).to_vec());
    let index_hash = ctx.upload_blob(cast_slice::<[u32; 3], u8>(&IDXS).to_vec());

    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
    let root = tree.create(None).expect("create");
    let meta = tree.get_meta(root).expect("get meta");

    reconcile_collider(
        &meta,
        ColliderAttr::Trimesh {
            vertices: ByteArray::<32>::new(*vertex_hash.as_bytes()),
            indices: ByteArray::<32>::new(*index_hash.as_bytes()),
        },
    );

    ctx.tick_until(|world| world.query::<&Collider>().iter(world).next().is_some());
}

#[traced_test]
#[rstest]
fn test_collider_scale_zero_does_not_panic(mut ctx: TestContext) {
    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
    let root = tree.create(None).expect("create");
    let meta = tree.get_meta(root).expect("get meta");

    reconcile_collider(
        &meta,
        ColliderAttr::Cuboid {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
    );

    ctx.doc.commit();
    ctx.app.update();

    // Collider should be present with valid default scale.
    let world = ctx.app.world_mut();
    let prim_ent = world
        .query::<(Entity, &Collider)>()
        .iter(world)
        .map(|(e, _)| e)
        .next()
        .expect("collider entity");

    // Give the prim entity a Transform with zero scale.
    world
        .entity_mut(prim_ent)
        .insert(Transform::from_scale(Vec3::ZERO));

    ctx.app.update();

    // watch_collider_scale should have removed Collider and stashed it.
    let world = ctx.app.world_mut();
    assert!(
        world.entity(prim_ent).get::<Collider>().is_none(),
        "Collider should be removed when scale is zero"
    );
    assert!(
        world.entity(prim_ent).get::<DisabledCollider>().is_some(),
        "DisabledCollider should hold the stashed collider"
    );

    // Restore a valid scale.
    world
        .entity_mut(prim_ent)
        .insert(Transform::from_scale(Vec3::ONE));

    ctx.app.update();
    std::thread::sleep(Duration::from_millis(100));
    ctx.app.update();

    let world = ctx.app.world_mut();
    assert!(
        world.entity(prim_ent).get::<Collider>().is_some(),
        "Collider should be restored when scale becomes valid"
    );
    assert!(
        world.entity(prim_ent).get::<DisabledCollider>().is_none(),
        "DisabledCollider should be removed after restore"
    );
}

fn reconcile_collider(meta: &loro::LoroMap, attr: ColliderAttr) {
    let prim = PrimMeta {
        attributes: Some(Attributes {
            collider: Some(attr),
            ..Default::default()
        }),
        ..Default::default()
    };
    prim.reconcile(RootReconciler::new(meta.clone()))
        .expect("reconcile");
}
