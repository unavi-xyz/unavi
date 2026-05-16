use avian3d::prelude::{AngularDamping, Friction, LinearDamping, Mass, Restitution, RigidBody};
use hsd::{
    HSD_CONTAINER_ID, PrimMeta,
    attributes::{Attribute, Attributes, attributes_map, rigid_body::{RigidBodyAttr, RigidBodyKind}},
};
use lorosurgeon::{MaybeMissing, Reconcile, reconcile::RootReconciler};
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

#[traced_test]
#[rstest]
fn test_rigid_body_lifecycle(mut ctx: TestContext) {
    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
    let root = tree.create(None).expect("create");
    let meta = tree.get_meta(root).expect("get meta");

    reconcile_rigid_body(&meta, RigidBodyAttr { kind: RigidBodyKind::Dynamic, ..Default::default() });

    ctx.doc.commit();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut q = world.query::<&RigidBody>();
    let rb = q.iter(world).next().expect("RigidBody expected");
    assert!(rb.is_dynamic());

    let attrs = attributes_map(&meta).expect("attributes map");
    attrs.delete(RigidBodyAttr::KEY).expect("delete");

    ctx.doc.commit();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut q = world.query::<&RigidBody>();
    assert!(q.iter(world).next().is_none(), "RigidBody should be removed");
}

#[traced_test]
#[rstest]
fn test_rigid_body_kinds(mut ctx: TestContext) {
    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);

    let static_prim = tree.create(None).expect("create");
    reconcile_rigid_body(
        &tree.get_meta(static_prim).expect("meta"),
        RigidBodyAttr { kind: RigidBodyKind::Static, ..Default::default() },
    );

    let kinematic_prim = tree.create(None).expect("create");
    reconcile_rigid_body(
        &tree.get_meta(kinematic_prim).expect("meta"),
        RigidBodyAttr { kind: RigidBodyKind::Kinematic, ..Default::default() },
    );

    ctx.doc.commit();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut q = world.query::<&RigidBody>();
    let bodies: Vec<&RigidBody> = q.iter(world).collect();
    assert_eq!(bodies.len(), 2);
    assert!(bodies.iter().any(|rb| rb.is_static()), "expected Static");
    assert!(bodies.iter().any(|rb| rb.is_kinematic()), "expected Kinematic");
}

#[traced_test]
#[rstest]
fn test_rigid_body_props(mut ctx: TestContext) {
    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
    let root = tree.create(None).expect("create");
    let meta = tree.get_meta(root).expect("get meta");

    reconcile_rigid_body(
        &meta,
        RigidBodyAttr {
            kind: RigidBodyKind::Dynamic,
            friction: MaybeMissing::Present(0.5),
            restitution: MaybeMissing::Present(0.3),
            mass: MaybeMissing::Present(2.0),
            linear_damping: MaybeMissing::Present(0.1),
            angular_damping: MaybeMissing::Present(0.2),
        },
    );

    ctx.doc.commit();
    ctx.app.update();

    let world = ctx.app.world_mut();

    let mut q = world.query::<&Friction>();
    let f = q.iter(world).next().expect("Friction");
    assert!((f.dynamic_coefficient - 0.5).abs() < 1e-5);

    let mut q = world.query::<&Restitution>();
    let r = q.iter(world).next().expect("Restitution");
    assert!((r.coefficient - 0.3).abs() < 1e-5);

    let mut q = world.query::<&Mass>();
    let m = q.iter(world).next().expect("Mass");
    assert!((m.0 - 2.0).abs() < 1e-5);

    let mut q = world.query::<&LinearDamping>();
    let ld = q.iter(world).next().expect("LinearDamping");
    assert!((ld.0 - 0.1).abs() < 1e-5);

    let mut q = world.query::<&AngularDamping>();
    let ad = q.iter(world).next().expect("AngularDamping");
    assert!((ad.0 - 0.2).abs() < 1e-5);
}

#[traced_test]
#[rstest]
fn test_rigid_body_invalid_mass(mut ctx: TestContext) {
    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);

    for bad_mass in [0.0_f64, -1.0, f64::NAN] {
        let root = tree.create(None).expect("create");
        let meta = tree.get_meta(root).expect("get meta");

        reconcile_rigid_body(
            &meta,
            RigidBodyAttr {
                kind: RigidBodyKind::Dynamic,
                mass: MaybeMissing::Present(bad_mass),
                ..Default::default()
            },
        );

        ctx.doc.commit();
        ctx.app.update();

        let world = ctx.app.world_mut();
        let mut q_mass = world.query::<&Mass>();
        assert!(q_mass.iter(world).next().is_none(), "Mass should not be inserted for invalid value {bad_mass}");
        let mut q_rb = world.query::<&RigidBody>();
        assert!(q_rb.iter(world).next().is_some(), "RigidBody should still be present");

        assert!(logs_contain("mass must be finite and > 0"));
    }
}

fn reconcile_rigid_body(meta: &loro::LoroMap, attr: RigidBodyAttr) {
    let prim = PrimMeta {
        attributes: MaybeMissing::Present(Attributes {
            rigid_body: MaybeMissing::Present(attr),
            ..Default::default()
        }),
        ..Default::default()
    };
    prim.reconcile(RootReconciler::new(meta.clone()))
        .expect("reconcile");
}
