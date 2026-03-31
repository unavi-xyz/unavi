use bevy_hsd::{
    HsdNodePhysics, NodeId,
    data::{HsdCollider, HsdNodeData, HsdRigidBody},
};
use loro::{LoroTree, TreeParentId};
use loro_surgeon::Reconcile;

mod common;
use common::TestHarness;

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

#[test]
fn collider_inserts_hsd_node_physics() {
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

    let mut q = h.app.world_mut().query::<(&NodeId, &HsdNodePhysics)>();
    let (_, physics) = q
        .iter(h.app.world())
        .next()
        .expect("HsdNodePhysics on node");
    assert!(physics.collider.is_some(), "collider should be set");
}

#[test]
fn rigid_body_inserts_hsd_node_physics() {
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

    let mut q = h.app.world_mut().query::<(&NodeId, &HsdNodePhysics)>();
    let (_, physics) = q
        .iter(h.app.world())
        .next()
        .expect("HsdNodePhysics on node");
    assert!(physics.rigid_body.is_some(), "rigid_body should be set");

    assert!(
        h.app
            .world()
            .get::<bevy_hsd::HsdChildren>(h.doc_entity)
            .is_some()
    );
}

#[test]
fn node_without_physics_has_empty_hsd_node_physics() {
    let mut h = TestHarness::new();
    add_node_with_data(&h, HsdNodeData::default());
    h.commit_and_update();

    // HsdNodePhysics is always inserted on node entities; fields should be None.
    let mut q = h.app.world_mut().query::<(&NodeId, &HsdNodePhysics)>();
    let (_, physics) = q
        .iter(h.app.world())
        .next()
        .expect("node with HsdNodePhysics");
    assert!(physics.collider.is_none(), "collider should be None");
    assert!(physics.rigid_body.is_none(), "rigid_body should be None");
}
