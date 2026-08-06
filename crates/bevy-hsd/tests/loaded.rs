use avian3d::prelude::Collider;
use bevy::prelude::*;
use bevy_hsd::loaded::{
    HsdLoaded,
    HsdSnapshotDrained,
};
use bytemuck::cast_slice;
use hsd::attributes::{
    collider::ColliderAttr,
    slots,
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

fn has<C: Component>(world: &mut World) -> bool {
    world.query::<&C>().iter(world).next().is_some()
}

#[traced_test]
#[rstest]
fn test_loaded_when_no_blob_work(mut ctx: TestContext) {
    let root = ctx.create_prim();
    ctx.set_attr(
        root,
        &ColliderAttr::Cuboid {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
    );

    ctx.tick_until(has::<HsdLoaded>);
}

#[traced_test]
#[rstest]
fn test_not_loaded_while_collider_broken(mut ctx: TestContext) {
    let root = ctx.create_prim();

    // Garbage bytes for a trimesh: the collider cannot be built, so readiness
    // must never fire while it is missing.
    ctx.set_attr(root, &ColliderAttr::Trimesh);
    ctx.set_slot(root, slots::COLLIDER_VERTICES, b"not-vertices".to_vec());
    ctx.set_slot(root, slots::COLLIDER_INDICES, b"not-indices".to_vec());

    for _ in 0..16 {
        ctx.app.update();
    }

    let world = ctx.app.world_mut();
    assert!(
        has::<HsdSnapshotDrained>(world),
        "the first event batch should be drained"
    );
    assert!(
        !has::<HsdLoaded>(world),
        "HsdLoaded must not fire while a collider is still broken"
    );
}

#[traced_test]
#[rstest]
fn test_loaded_after_collider_built(mut ctx: TestContext) {
    let vertices = cast_slice::<[f32; 3], u8>(&VERTS).to_vec();
    let indices = cast_slice::<[u32; 3], u8>(&IDXS).to_vec();

    let root = ctx.create_prim();
    ctx.set_attr(root, &ColliderAttr::Trimesh);
    ctx.set_slot(root, slots::COLLIDER_VERTICES, vertices);
    ctx.set_slot(root, slots::COLLIDER_INDICES, indices);

    ctx.tick_until(has::<Collider>);
    ctx.tick_until(has::<HsdLoaded>);
}
