use bevy::prelude::*;
use bevy_hsd::{HsdChild, NodeId};
use loro::LoroMap;

mod common;
use common::TestHarness;

fn add_mesh(harness: &TestHarness, id: &str) {
    harness
        .doc
        .get_map("hsd")
        .get_or_create_container("meshes", LoroMap::new())
        .expect("meshes map")
        .get_or_create_container(id, LoroMap::new())
        .expect("mesh map entry");
}

#[test]
fn mesh_entity_spawns() {
    let mut h = TestHarness::new();
    add_mesh(&h, "mesh-0");
    h.commit_and_update();

    let mut with_node = h.app.world_mut().query_filtered::<Entity, With<NodeId>>();
    let node_count = with_node.iter(h.app.world()).count();

    let mut with_child = h.app.world_mut().query_filtered::<Entity, With<HsdChild>>();
    let child_count = with_child.iter(h.app.world()).count();

    assert_eq!(node_count, 0, "no node entities expected");
    assert_eq!(child_count, 1, "one mesh entity expected");
}

#[test]
fn two_meshes_spawn_two_entities() {
    let mut h = TestHarness::new();
    add_mesh(&h, "mesh-a");
    add_mesh(&h, "mesh-b");
    h.commit_and_update();

    let mut q = h.app.world_mut().query_filtered::<Entity, With<HsdChild>>();
    assert_eq!(
        q.iter(h.app.world()).count(),
        2,
        "two mesh entities expected"
    );
}

#[test]
fn mesh_removed() {
    let mut h = TestHarness::new();
    add_mesh(&h, "mesh-0");
    h.commit_and_update();

    let mut q = h.app.world_mut().query_filtered::<Entity, With<HsdChild>>();
    assert_eq!(q.iter(h.app.world()).count(), 1);

    h.doc
        .get_map("hsd")
        .get_or_create_container("meshes", LoroMap::new())
        .expect("meshes map")
        .delete("mesh-0")
        .expect("delete mesh");
    h.commit_and_update();

    let mut q2 = h.app.world_mut().query_filtered::<Entity, With<HsdChild>>();
    assert_eq!(
        q2.iter(h.app.world()).count(),
        0,
        "mesh entity should be gone"
    );

    assert!(
        h.app
            .world()
            .get::<bevy_hsd::HsdChildren>(h.doc_entity)
            .is_none()
    );
}
