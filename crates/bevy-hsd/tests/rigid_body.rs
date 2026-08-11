use avian3d::prelude::{
    AngularDamping,
    Friction,
    LinearDamping,
    Mass,
    Restitution,
    RigidBody,
};
use bevy::prelude::*;
use hsd::attributes::rigid_body::{
    RigidBodyAttr,
    RigidBodyKind,
};
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

#[traced_test]
#[rstest]
fn test_rigid_body_lifecycle(mut ctx: TestContext) {
    let root = ctx.create_prim();
    ctx.set_attr(
        root,
        &RigidBodyAttr {
            kind: Some(RigidBodyKind::Dynamic),
            ..Default::default()
        },
    );

    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut q = world.query::<&RigidBody>();
    let rb = q.iter(world).next().expect("RigidBody expected");
    assert!(rb.is_dynamic());

    ctx.remove_attr::<RigidBodyAttr>(root);
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut q = world.query::<&RigidBody>();
    assert!(
        q.iter(world).next().is_none(),
        "RigidBody should be removed"
    );
}

#[traced_test]
#[rstest]
fn test_rigid_body_kinds(mut ctx: TestContext) {
    let static_prim = ctx.create_prim();
    ctx.set_attr(
        static_prim,
        &RigidBodyAttr {
            kind: Some(RigidBodyKind::Static),
            ..Default::default()
        },
    );

    let kinematic_prim = ctx.create_prim();
    ctx.set_attr(
        kinematic_prim,
        &RigidBodyAttr {
            kind: Some(RigidBodyKind::Kinematic),
            ..Default::default()
        },
    );

    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut q = world.query::<&RigidBody>();
    let bodies: Vec<&RigidBody> = q.iter(world).collect();
    assert_eq!(bodies.len(), 2);
    assert!(bodies.iter().any(|rb| rb.is_static()), "expected Static");
    assert!(
        bodies.iter().any(|rb| rb.is_kinematic()),
        "expected Kinematic"
    );
}

#[traced_test]
#[rstest]
fn test_rigid_body_props(mut ctx: TestContext) {
    let root = ctx.create_prim();
    ctx.set_attr(
        root,
        &RigidBodyAttr {
            kind:            Some(RigidBodyKind::Dynamic),
            friction:        Some(0.5),
            restitution:     Some(0.3),
            mass:            Some(2.0),
            linear_damping:  Some(0.1),
            angular_damping: Some(0.2),
        },
    );

    ctx.app.update();

    let world = ctx.app.world_mut();

    let mut q = world.query::<&Friction>();
    let f = q.iter(world).next().expect("Friction");
    assert!((f.dynamic_coefficient - 0.5).abs() < 1.0e-5);

    let mut q = world.query::<&Restitution>();
    let r = q.iter(world).next().expect("Restitution");
    assert!((r.coefficient - 0.3).abs() < 1.0e-5);

    let mut q = world.query::<&Mass>();
    let m = q.iter(world).next().expect("Mass");
    assert!((m.0 - 2.0).abs() < 1.0e-5);

    let mut q = world.query::<&LinearDamping>();
    let ld = q.iter(world).next().expect("LinearDamping");
    assert!((ld.0 - 0.1).abs() < 1.0e-5);

    let mut q = world.query::<&AngularDamping>();
    let ad = q.iter(world).next().expect("AngularDamping");
    assert!((ad.0 - 0.2).abs() < 1.0e-5);
}

#[traced_test]
#[rstest]
fn test_rigid_body_invalid_mass(mut ctx: TestContext) {
    for bad_mass in [0.0_f64, -1.0, f64::NAN] {
        let root = ctx.create_prim();
        ctx.set_attr(
            root,
            &RigidBodyAttr {
                kind: Some(RigidBodyKind::Dynamic),
                mass: Some(bad_mass),
                ..Default::default()
            },
        );

        ctx.app.update();

        let world = ctx.app.world_mut();
        let mut q_mass = world.query::<&Mass>();
        assert!(
            q_mass.iter(world).next().is_none(),
            "Mass should not be inserted for invalid value {bad_mass}"
        );
        let mut q_rb = world.query::<&RigidBody>();
        assert!(
            q_rb.iter(world).next().is_some(),
            "RigidBody should still be present"
        );

        assert!(logs_contain("mass must be finite and > 0"));
    }
}
