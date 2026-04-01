use avian3d::prelude::{AngularDamping, Collider, LinearDamping, RigidBody};
use bevy::prelude::Entity;
use bevy_hsd::{
    NodeId,
    data::{HsdCollider, HsdNodeData, HsdRigidBody},
};
use loro::{LoroTree, TreeParentId};
use loro_surgeon::Reconcile;

mod common;

use common::TestHarness;

const EPSILON: f32 = 1e-5;

fn add_node_with_data(harness: &TestHarness, data: HsdNodeData) {
    let nodes = harness
        .doc
        .get_map("hsd")
        .get_or_create_container("nodes", LoroTree::new())
        .expect("nodes tree");
    let tid = nodes.create(TreeParentId::Root).expect("create node");
    let meta = nodes.get_meta(tid).expect("node meta");
    data.reconcile(&meta).expect("reconcile node data");
}

fn node_entity(h: &mut TestHarness) -> Entity {
    h.app
        .world_mut()
        .query::<(Entity, &NodeId)>()
        .iter(h.app.world())
        .next()
        .map(|(e, _)| e)
        .expect("node entity")
}

#[test]
fn collider_cuboid_inserted() {
    let mut h = TestHarness::new();
    add_node_with_data(
        &h,
        HsdNodeData {
            collider: Some(HsdCollider::Cuboid {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            }),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let ent = node_entity(&mut h);
    assert!(
        h.app.world().get::<Collider>(ent).is_some(),
        "Collider component expected on node"
    );
}

#[test]
fn collider_capsule_inserted() {
    let mut h = TestHarness::new();
    add_node_with_data(
        &h,
        HsdNodeData {
            collider: Some(HsdCollider::Capsule {
                height: 2.0,
                radius: 0.5,
            }),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let ent = node_entity(&mut h);
    assert!(
        h.app.world().get::<Collider>(ent).is_some(),
        "Collider component expected on node"
    );
}

#[test]
fn collider_cylinder_inserted() {
    let mut h = TestHarness::new();
    add_node_with_data(
        &h,
        HsdNodeData {
            collider: Some(HsdCollider::Cylinder {
                height: 1.0,
                radius: 0.5,
            }),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let ent = node_entity(&mut h);
    assert!(
        h.app.world().get::<Collider>(ent).is_some(),
        "Collider component expected on node"
    );
}

#[test]
fn collider_sphere_inserted() {
    let mut h = TestHarness::new();
    add_node_with_data(
        &h,
        HsdNodeData {
            collider: Some(HsdCollider::Sphere(0.75)),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let ent = node_entity(&mut h);
    assert!(
        h.app.world().get::<Collider>(ent).is_some(),
        "Collider component expected on node"
    );
}

#[test]
fn rigid_body_dynamic_inserted() {
    let mut h = TestHarness::new();
    add_node_with_data(
        &h,
        HsdNodeData {
            rigid_body: Some(HsdRigidBody {
                kind: "dynamic".into(),
                ..Default::default()
            }),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let ent = node_entity(&mut h);
    let rb = h
        .app
        .world()
        .get::<RigidBody>(ent)
        .expect("RigidBody on node");
    assert_eq!(*rb, RigidBody::Dynamic);
}

#[test]
fn rigid_body_fixed_maps_to_kinematic() {
    let mut h = TestHarness::new();
    add_node_with_data(
        &h,
        HsdNodeData {
            rigid_body: Some(HsdRigidBody {
                kind: "fixed".into(),
                ..Default::default()
            }),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let ent = node_entity(&mut h);
    let rb = h
        .app
        .world()
        .get::<RigidBody>(ent)
        .expect("RigidBody on node");
    assert_eq!(*rb, RigidBody::Kinematic);
}

#[test]
fn rigid_body_dynamic_linear_damping() {
    let mut h = TestHarness::new();
    add_node_with_data(
        &h,
        HsdNodeData {
            rigid_body: Some(HsdRigidBody {
                kind: "dynamic".into(),
                linear_damping: Some(0.4),
                ..Default::default()
            }),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let ent = node_entity(&mut h);
    let damping = h
        .app
        .world()
        .get::<LinearDamping>(ent)
        .expect("LinearDamping on node");
    assert!(
        (damping.0 - 0.4).abs() < EPSILON,
        "linear_damping: {}",
        damping.0
    );
}

#[test]
fn rigid_body_dynamic_angular_damping() {
    let mut h = TestHarness::new();
    add_node_with_data(
        &h,
        HsdNodeData {
            rigid_body: Some(HsdRigidBody {
                kind: "dynamic".into(),
                angular_damping: Some(0.6),
                ..Default::default()
            }),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let ent = node_entity(&mut h);
    let damping = h
        .app
        .world()
        .get::<AngularDamping>(ent)
        .expect("AngularDamping on node");
    assert!(
        (damping.0 - 0.6).abs() < EPSILON,
        "angular_damping: {}",
        damping.0
    );
}

#[test]
fn node_without_physics_has_no_avian_components() {
    let mut h = TestHarness::new();
    add_node_with_data(&h, HsdNodeData::default());
    h.commit_and_update();

    let ent = node_entity(&mut h);
    assert!(
        h.app.world().get::<Collider>(ent).is_none(),
        "no Collider expected"
    );
    assert!(
        h.app.world().get::<RigidBody>(ent).is_none(),
        "no RigidBody expected"
    );
}
