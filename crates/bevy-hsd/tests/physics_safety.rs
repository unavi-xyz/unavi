//! Regression tests for hazards in the collider + rigid-body + xform
//! pipeline: avian-observer panics from uninitialised placeholders,
//! and prims spawning at the wrong location because `Position` ends up
//! out of sync with the script's intended `Transform`.

use avian3d::prelude::{Collider, Position, Rotation};
use bevy::prelude::*;
use bevy_hsd::attributes::collider::DisabledCollider;
use hsd::{
    HSD_CONTAINER_ID, PrimMeta,
    attributes::{
        Attributes,
        collider::ColliderAttr,
        rigid_body::{RigidBodyAttr, RigidBodyKind},
        xform::XformAttr,
    },
};
use loro_surgeon::{Reconcile, reconcile::RootReconciler};
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

/// One commit writes collider + rigid body + zero-scale xform — must not
/// panic on avian's placeholder `Position` / `Rotation`.
#[traced_test]
#[rstest]
#[case::static_body(RigidBodyKind::Static)]
#[case::dynamic(RigidBodyKind::Dynamic)]
#[case::kinematic(RigidBodyKind::Kinematic)]
fn collider_plus_rigid_body_plus_zero_scale_does_not_panic(
    mut ctx_physics: TestContext,
    #[case] kind: RigidBodyKind,
) {
    write_full_prim(
        &ctx_physics.doc,
        Some(ColliderAttr::Cuboid {
            x: 1.0,
            y: 0.5,
            z: 1.0,
        }),
        Some(RigidBodyAttr {
            kind: Some(kind),
            ..Default::default()
        }),
        Some(XformAttr {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [0.0, 0.0, 0.0],
        }),
    );

    ctx_physics.doc.commit();
    // The original panic fired during this update's command apply.
    ctx_physics.app.update();
    ctx_physics.app.update();

    // scale=0 → watch_collider_scale parks the collider as DisabledCollider.
    let world = ctx_physics.app.world_mut();
    let mut parked = world.query::<&DisabledCollider>();
    assert!(
        parked.iter(world).next().is_some(),
        "Collider should be parked while scale=0"
    );
}

/// Collider with no rigid body — placeholder hazard still applies.
#[traced_test]
#[rstest]
fn collider_without_rigid_body_does_not_panic(mut ctx_physics: TestContext) {
    write_full_prim(
        &ctx_physics.doc,
        Some(ColliderAttr::Sphere(0.5)),
        None,
        None,
    );

    ctx_physics.doc.commit();
    ctx_physics.app.update();
    ctx_physics.app.update();

    let world = ctx_physics.app.world_mut();
    let mut q = world.query::<(&Collider, &Position, &Rotation)>();
    let (_, pos, rot) = q.iter(world).next().expect("collider");
    assert!(pos.0.is_finite(), "Position must be finite, got {pos:?}");
    assert!(
        rot.x.is_finite() && rot.y.is_finite() && rot.z.is_finite() && rot.w.is_finite(),
        "Rotation must be finite, got {rot:?}"
    );
}

/// Collider with no xform attr — seed must cope with a default `Transform`.
#[traced_test]
#[rstest]
fn collider_without_xform_does_not_panic(mut ctx_physics: TestContext) {
    write_full_prim(
        &ctx_physics.doc,
        Some(ColliderAttr::Cylinder {
            height: 0.1,
            radius: 0.6,
        }),
        Some(RigidBodyAttr {
            kind: Some(RigidBodyKind::Static),
            ..Default::default()
        }),
        None,
    );

    ctx_physics.doc.commit();
    ctx_physics.app.update();
    ctx_physics.app.update();
}

/// A non-zero translation in the script's xform must survive avian's
/// `init_physics_transform` write-back; otherwise prims land at the origin.
#[traced_test]
#[rstest]
fn xform_translation_is_not_clobbered_by_init_physics_transform(mut ctx_physics: TestContext) {
    write_full_prim(
        &ctx_physics.doc,
        Some(ColliderAttr::Cuboid {
            x: 0.2,
            y: 0.2,
            z: 0.2,
        }),
        Some(RigidBodyAttr {
            kind: Some(RigidBodyKind::Static),
            ..Default::default()
        }),
        Some(XformAttr {
            translation: [5.0, 2.0, -3.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }),
    );

    ctx_physics.doc.commit();
    ctx_physics.app.update();
    ctx_physics.app.update();

    let world = ctx_physics.app.world_mut();
    let mut q = world.query::<&Transform>();
    let transform = q.iter(world).next().expect("transform");
    assert!(
        (transform.translation - Vec3::new(5.0, 2.0, -3.0)).length() < 1.0e-4,
        "Transform.translation should be the value the script set, got {:?}",
        transform.translation
    );
}

/// Promoting a parked (scale-0) collider back to active when scale flips
/// to a valid value must not panic.
#[traced_test]
#[rstest]
fn scale_zero_then_nonzero_restores_collider(mut ctx_physics: TestContext) {
    let tree = ctx_physics.doc.get_tree(&*HSD_CONTAINER_ID);
    let root = tree.create(None).expect("create");
    let meta = tree.get_meta(root).expect("get meta");

    let prim = PrimMeta {
        attributes: Some(Attributes {
            collider: Some(ColliderAttr::Cuboid {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            }),
            rigid_body: Some(RigidBodyAttr {
                kind: Some(RigidBodyKind::Static),
                ..Default::default()
            }),
            xform: Some(XformAttr {
                translation: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [0.0, 0.0, 0.0],
            }),
            ..Default::default()
        }),
        ..Default::default()
    };
    prim.reconcile(RootReconciler::new(meta.clone()))
        .expect("reconcile");

    ctx_physics.doc.commit();
    ctx_physics.app.update();
    ctx_physics.app.update();

    // Now flip scale to 1.
    let prim_active = PrimMeta {
        attributes: Some(Attributes {
            collider: Some(ColliderAttr::Cuboid {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            }),
            rigid_body: Some(RigidBodyAttr {
                kind: Some(RigidBodyKind::Static),
                ..Default::default()
            }),
            xform: Some(XformAttr {
                translation: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
            }),
            ..Default::default()
        }),
        ..Default::default()
    };
    prim_active
        .reconcile(RootReconciler::new(meta))
        .expect("reconcile");

    ctx_physics.doc.commit();
    ctx_physics.app.update();
    ctx_physics.app.update();

    let world = ctx_physics.app.world_mut();
    let mut q = world.query::<(&Collider, &Position, &Rotation)>();
    assert!(
        q.iter(world).next().is_some(),
        "Collider must be active after scale-up"
    );
}

/// A child prim with collider + rigid body whose parent is translated:
/// `Position` is global in avian, so it must be (parent + local), not local.
#[traced_test]
#[rstest]
fn child_of_translated_parent_has_global_position(mut ctx_physics: TestContext) {
    let tree = ctx_physics.doc.get_tree(&*HSD_CONTAINER_ID);
    let parent = tree.create(None).expect("create parent");
    let parent_meta = tree.get_meta(parent).expect("parent meta");
    PrimMeta {
        attributes: Some(Attributes {
            xform: Some(XformAttr {
                translation: [0.0, 0.0, -10.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
    .reconcile(RootReconciler::new(parent_meta))
    .expect("reconcile parent");

    let child = tree.create(Some(parent)).expect("create child");
    let child_meta = tree.get_meta(child).expect("child meta");
    PrimMeta {
        attributes: Some(Attributes {
            collider: Some(ColliderAttr::Cuboid {
                x: 0.2,
                y: 0.2,
                z: 0.2,
            }),
            rigid_body: Some(RigidBodyAttr {
                kind: Some(RigidBodyKind::Static),
                ..Default::default()
            }),
            xform: Some(XformAttr {
                translation: [-1.5, 0.5, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
    .reconcile(RootReconciler::new(child_meta))
    .expect("reconcile child");

    ctx_physics.doc.commit();
    ctx_physics.app.update();
    ctx_physics.app.update();

    let world = ctx_physics.app.world_mut();
    let mut q = world.query::<(&Position, &Transform)>();
    let mut found = false;
    for (pos, transform) in q.iter(world) {
        // Match the child by its local translation.
        if (transform.translation - Vec3::new(-1.5, 0.5, 0.0)).length() < 1.0e-3 {
            assert!(
                (pos.0 - Vec3::new(-1.5, 0.5, -10.0)).length() < 1.0e-3,
                "Position must be global (parent + local), got {:?}",
                pos.0
            );
            found = true;
        }
    }
    assert!(found, "child entity not found");
}

/// A child prim that has no xform attr of its own under a parent that's
/// scaled to zero: must end up parked, not with NaN `Transform` (which
/// `init_physics_transform`'s `reparented_to` on a degenerate parent
/// would produce, leaving the mesh invisible forever once the parent
/// scales back up).
#[traced_test]
#[rstest]
fn no_xform_child_of_zero_scale_parent_has_finite_transform(mut ctx_physics: TestContext) {
    let tree = ctx_physics.doc.get_tree(&*HSD_CONTAINER_ID);
    let parent = tree.create(None).expect("create parent");
    let parent_meta = tree.get_meta(parent).expect("parent meta");
    PrimMeta {
        attributes: Some(Attributes {
            xform: Some(XformAttr {
                translation: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [0.0, 0.0, 0.0],
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
    .reconcile(RootReconciler::new(parent_meta))
    .expect("reconcile parent");

    let child = tree.create(Some(parent)).expect("create child");
    let child_meta = tree.get_meta(child).expect("child meta");
    PrimMeta {
        attributes: Some(Attributes {
            collider: Some(ColliderAttr::Cuboid {
                x: 0.5,
                y: 0.5,
                z: 0.5,
            }),
            rigid_body: Some(RigidBodyAttr {
                kind: Some(RigidBodyKind::Static),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
    .reconcile(RootReconciler::new(child_meta))
    .expect("reconcile child");

    ctx_physics.doc.commit();
    ctx_physics.app.update();
    ctx_physics.app.update();

    let world = ctx_physics.app.world_mut();
    let mut q = world.query::<(&DisabledCollider, &Transform)>();
    let (_, transform) = q.iter(world).next().expect("collider parked");
    assert!(
        transform.translation.is_finite()
            && transform.rotation.x.is_finite()
            && transform.rotation.y.is_finite()
            && transform.rotation.z.is_finite()
            && transform.rotation.w.is_finite(),
        "Transform must stay finite even under a degenerate parent, got {transform:?}"
    );
}

fn write_full_prim(
    doc: &loro::LoroDoc,
    collider: Option<ColliderAttr>,
    rigid_body: Option<RigidBodyAttr>,
    xform: Option<XformAttr>,
) {
    let tree = doc.get_tree(&*HSD_CONTAINER_ID);
    let root = tree.create(None).expect("create");
    let meta = tree.get_meta(root).expect("get meta");

    let prim = PrimMeta {
        attributes: Some(Attributes {
            collider,
            rigid_body,
            xform,
            ..Default::default()
        }),
        ..Default::default()
    };
    prim.reconcile(RootReconciler::new(meta))
        .expect("reconcile");
}
