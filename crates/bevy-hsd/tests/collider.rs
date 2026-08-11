use avian3d::prelude::Collider;
use bevy::prelude::*;
use bevy_hsd::attributes::collider::HsdCollider;
use bytemuck::cast_slice;
use hsd::attributes::{
    collider::ColliderAttr,
    slots,
};
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

#[traced_test]
#[rstest]
fn test_collider_lifecycle(mut ctx: TestContext) {
    let root = ctx.create_prim();
    ctx.set_attr(root, &ColliderAttr::Sphere(0.5));

    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut q = world.query::<(&HsdCollider, &Collider)>();
    assert!(
        q.iter(world).next().is_some(),
        "HsdCollider + Collider expected"
    );

    ctx.remove_attr::<ColliderAttr>(root);
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
        let root = ctx.create_prim();
        ctx.set_attr(root, &ColliderAttr::Sphere(bad_r));

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
        let root = ctx.create_prim();
        ctx.set_attr(root, &ColliderAttr::Cuboid { x, y, z });

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

/// The shape and its buffers are separate entries and arrive in no fixed
/// order, so the collider must build once both halves are present.
#[traced_test]
#[rstest]
fn test_collider_trimesh(#[from(ctx)] mut ctx: TestContext) {
    const VERTS: [[f32; 3]; 4] = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ];
    const IDXS: [[u32; 3]; 4] = [[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]];

    let vertices = cast_slice::<[f32; 3], u8>(&VERTS).to_vec();
    let indices = cast_slice::<[u32; 3], u8>(&IDXS).to_vec();

    let root = ctx.create_prim();
    ctx.set_slot(root, slots::COLLIDER_VERTICES, vertices);
    ctx.set_attr(root, &ColliderAttr::Trimesh);
    ctx.set_slot(root, slots::COLLIDER_INDICES, indices);

    ctx.tick_until(|world| world.query::<&Collider>().iter(world).next().is_some());
}
