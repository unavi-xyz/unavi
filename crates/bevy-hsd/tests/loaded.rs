use avian3d::prelude::Collider;
use bevy::prelude::*;
use bevy_hsd::loaded::{
    HsdLoaded,
    HsdSnapshotDrained,
};
use bytemuck::cast_slice;
use hsd::{
    HSD_CONTAINER_ID,
    PrimMeta,
    attributes::{
        Attributes,
        collider::ColliderAttr,
    },
};
use loro_surgeon::{
    Reconcile,
    bytes::ByteArray,
    reconcile::RootReconciler,
};
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

const VERTS: [[f32; 3]; 4] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
];
const IDXS: [[u32; 3]; 4] = [[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]];

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

fn has<C: Component>(world: &mut World) -> bool {
    world.query::<&C>().iter(world).next().is_some()
}

#[traced_test]
#[rstest]
fn test_loaded_when_no_blob_work(mut ctx: TestContext) {
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

    ctx.tick_until(has::<HsdLoaded>);
}

#[traced_test]
#[rstest]
fn test_not_loaded_while_blob_pending(mut ctx: TestContext) {
    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
    let root = tree.create(None).expect("create");
    let meta = tree.get_meta(root).expect("get meta");

    // Blobs are never served (no wds), so the loader stays in-flight forever.
    reconcile_collider(
        &meta,
        ColliderAttr::Trimesh {
            vertices: ByteArray::<32>::new([1; 32]),
            indices:  ByteArray::<32>::new([2; 32]),
        },
    );

    ctx.doc.commit();
    for _ in 0..16 {
        ctx.app.update();
    }

    let world = ctx.app.world_mut();
    assert!(
        has::<HsdSnapshotDrained>(world),
        "snapshot should be drained"
    );
    assert!(
        !has::<HsdLoaded>(world),
        "HsdLoaded must not fire while a collider blob is still loading"
    );
}

#[traced_test]
#[rstest]
fn test_loaded_after_blob_resolves(#[from(ctx_wds)] mut ctx: TestContext) {
    let vertex_hash = ctx.upload_blob(cast_slice::<[f32; 3], u8>(&VERTS).to_vec());
    let index_hash = ctx.upload_blob(cast_slice::<[u32; 3], u8>(&IDXS).to_vec());

    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
    let root = tree.create(None).expect("create");
    let meta = tree.get_meta(root).expect("get meta");

    reconcile_collider(
        &meta,
        ColliderAttr::Trimesh {
            vertices: ByteArray::<32>::new(*vertex_hash.as_bytes()),
            indices:  ByteArray::<32>::new(*index_hash.as_bytes()),
        },
    );

    ctx.tick_until(has::<Collider>);
    ctx.tick_until(has::<HsdLoaded>);
}
