use avian3d::prelude::Collider;
use bevy::prelude::*;
use bytemuck::cast_slice;
use hsd::attributes::collider::ColliderKind;
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

#[traced_test]
#[rstest]
fn test_collider_lifecycle(mut ctx: TestContext) {
    let root = ctx.create_prim();
    ctx.set_attr(root, &ColliderKind::Sphere(0.5));

    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut q = world.query::<&Collider>();
    assert!(q.iter(world).next().is_some(), "Collider expected");

    ctx.remove_attr::<ColliderKind>(root);
    ctx.app.update();

    let world = ctx.app.world_mut();
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
        ctx.set_attr(root, &ColliderKind::Sphere(bad_r));

        ctx.app.update();

        let world = ctx.app.world_mut();
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
        ctx.set_attr(root, &ColliderKind::Cuboid { x, y, z });

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
    ctx.set_attr(root, &ColliderKind::Trimesh);
    ctx.set_collider_vertices(root, vertices);
    ctx.set_collider_indices(root, indices);

    ctx.tick_until(|world| world.query::<&Collider>().iter(world).next().is_some());
}

/// A scene cannot grow the solver's collider count past
/// [`unavi_physics::PhysicsLimits::max_colliders`]: once two prims ask for a
/// collider in the same frame and the cap only has room for one, one of the
/// two is refused (which one is unspecified).
#[traced_test]
#[rstest]
fn test_collider_limit_refuses_excess_colliders(mut ctx: TestContext) {
    ctx.app.insert_resource(unavi_physics::PhysicsLimits {
        max_bodies:    usize::MAX,
        max_colliders: 1,
    });

    let first = ctx.create_prim();
    ctx.set_attr(first, &ColliderKind::Sphere(0.5));
    let second = ctx.create_prim();
    ctx.set_attr(second, &ColliderKind::Sphere(0.5));

    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut q = world.query::<&Collider>();
    assert_eq!(
        q.iter(world).count(),
        1,
        "the scene's second collider should have been refused"
    );

    assert!(logs_contain("scene exceeded the collider limit"));
}

/// A collider Rust code spawns directly (a local agent's rig, a portal
/// sensor) has no [`bevy_hsd::prim::Prim`] marker. It must not count against
/// a scene's own collider budget.
#[traced_test]
#[rstest]
fn test_collider_limit_ignores_non_prim_colliders(mut ctx: TestContext) {
    ctx.app.insert_resource(unavi_physics::PhysicsLimits {
        max_bodies:    usize::MAX,
        max_colliders: 1,
    });

    // Fills the entire budget with a collider that is not a scene prim.
    let non_prim = ctx.app.world_mut().spawn(Collider::sphere(0.5)).id();

    let prim = ctx.create_prim();
    ctx.set_attr(prim, &ColliderKind::Sphere(0.5));

    ctx.app.update();

    let world = ctx.app.world_mut();
    assert!(
        world.get::<Collider>(non_prim).is_some(),
        "a non-prim collider must never be refused on the scene's behalf"
    );
    let mut q = world.query::<(&Collider, &bevy_hsd::prim::Prim)>();
    assert_eq!(
        q.iter(world).count(),
        1,
        "the scene's own prim must not be refused because of a collider that is not a prim"
    );
}
