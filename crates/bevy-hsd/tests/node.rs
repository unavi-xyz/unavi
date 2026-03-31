use bevy::prelude::*;
use bevy_hsd::{HsdChild, HsdChildren, MeshRef, NodeId, data::HsdNodeData};
use loro::{LoroMap, LoroTree, TreeParentId};
use loro_surgeon::Reconcile;

mod common;
use common::TestHarness;

fn add_root_node(harness: &TestHarness) -> loro::TreeID {
    harness
        .doc
        .get_map("hsd")
        .get_or_create_container("nodes", LoroTree::new())
        .expect("nodes tree")
        .create(TreeParentId::Root)
        .expect("create root node")
}

fn set_node_data(harness: &TestHarness, tid: loro::TreeID, data: HsdNodeData) {
    let nodes = harness
        .doc
        .get_map("hsd")
        .get_or_create_container("nodes", LoroTree::new())
        .expect("nodes tree");
    let meta = nodes.get_meta(tid).expect("node meta");
    data.reconcile(&meta).expect("reconcile node data");
}

#[test]
fn node_spawns() {
    let mut h = TestHarness::new();
    add_root_node(&h);
    h.commit_and_update();

    let mut q = h.app.world_mut().query::<&NodeId>();
    assert_eq!(q.iter(h.app.world()).count(), 1, "one node entity expected");
}

#[test]
fn node_name_set() {
    let mut h = TestHarness::new();
    let tid = add_root_node(&h);
    set_node_data(
        &h,
        tid,
        HsdNodeData {
            name: Some("my-node".into()),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let mut q = h.app.world_mut().query::<(&NodeId, &Name)>();
    let (_, name) = q.iter(h.app.world()).next().expect("node with Name");
    assert_eq!(name.as_str(), "my-node");
}

#[test]
fn node_transform_set() {
    let mut h = TestHarness::new();
    let tid = add_root_node(&h);
    set_node_data(
        &h,
        tid,
        HsdNodeData {
            translation: Some(vec![1.0, 2.0, 3.0]),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let mut q = h.app.world_mut().query::<(&NodeId, &Transform)>();
    let (_, t) = q.iter(h.app.world()).next().expect("node with Transform");
    let expected = Vec3::new(1.0, 2.0, 3.0);
    assert!(
        (t.translation - expected).length() < 1e-5,
        "translation mismatch: {:?}",
        t.translation
    );
}

#[test]
fn node_mesh_ref_set() {
    let mut h = TestHarness::new();

    // Create a mesh entry so the mesh entity also spawns.
    h.doc
        .get_map("hsd")
        .get_or_create_container("meshes", LoroMap::new())
        .expect("meshes map")
        .get_or_create_container("mesh-0", LoroMap::new())
        .expect("mesh-0 map");

    let tid = add_root_node(&h);
    set_node_data(
        &h,
        tid,
        HsdNodeData {
            mesh: Some("mesh-0".into()),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let mut q = h.app.world_mut().query::<(&NodeId, &MeshRef)>();
    assert!(q.iter(h.app.world()).next().is_some(), "MeshRef expected");
}

#[test]
fn node_parent_child() {
    let mut h = TestHarness::new();

    let nodes = h
        .doc
        .get_map("hsd")
        .get_or_create_container("nodes", LoroTree::new())
        .expect("nodes tree");
    let parent_tid = nodes.create(TreeParentId::Root).expect("create parent");
    nodes
        .create(TreeParentId::Node(parent_tid))
        .expect("create child");
    h.commit_and_update();

    let mut q = h.app.world_mut().query::<&NodeId>();
    assert_eq!(
        q.iter(h.app.world()).count(),
        2,
        "two node entities expected"
    );

    let mut q2 = h.app.world_mut().query::<(&NodeId, &ChildOf)>();
    assert_eq!(q2.iter(h.app.world()).count(), 1, "one child node expected");
}

#[test]
fn node_despawns() {
    let mut h = TestHarness::new();
    let tid = add_root_node(&h);
    h.commit_and_update();

    let mut q = h.app.world_mut().query::<&NodeId>();
    assert_eq!(q.iter(h.app.world()).count(), 1);

    h.doc
        .get_map("hsd")
        .get_or_create_container("nodes", LoroTree::new())
        .expect("nodes tree")
        .delete(tid)
        .expect("delete node");
    h.commit_and_update();

    let mut q2 = h.app.world_mut().query::<&NodeId>();
    assert_eq!(
        q2.iter(h.app.world()).count(),
        0,
        "node should be despawned"
    );
}

#[test]
fn hsd_children_tracked() {
    let mut h = TestHarness::new();
    add_root_node(&h);
    h.commit_and_update();

    let mut q = h.app.world_mut().query_filtered::<Entity, With<HsdChild>>();
    assert!(
        q.iter(h.app.world()).count() >= 1,
        "at least one HsdChild entity expected"
    );

    assert!(
        h.app.world().get::<HsdChildren>(h.doc_entity).is_some(),
        "HsdChildren on doc entity"
    );
}

#[test]
fn hsd_doc_despawn_removes_children() {
    let mut h = TestHarness::new();
    add_root_node(&h);
    h.commit_and_update();

    let mut q = h.app.world_mut().query_filtered::<Entity, With<HsdChild>>();
    assert_eq!(q.iter(h.app.world()).count(), 1);

    h.app.world_mut().commands().entity(h.doc_entity).despawn();
    h.app.update();

    let mut q2 = h.app.world_mut().query_filtered::<Entity, With<HsdChild>>();
    assert_eq!(
        q2.iter(h.app.world()).count(),
        0,
        "HsdChild entities should be despawned with the doc entity"
    );
}
