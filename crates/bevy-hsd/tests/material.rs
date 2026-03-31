use bevy::prelude::*;
use bevy_hsd::{
    HsdChild, MaterialRef, NodeId,
    data::{HsdMaterial, HsdNodeData},
};
use loro::{LoroMap, LoroTree, TreeParentId};
use loro_surgeon::Reconcile;

mod common;
use common::TestHarness;

fn add_material(harness: &TestHarness, id: &str) {
    harness
        .doc
        .get_map("hsd")
        .get_or_create_container("materials", LoroMap::new())
        .expect("materials map")
        .get_or_create_container(id, LoroMap::new())
        .expect("material map entry");
}

fn set_material(harness: &TestHarness, id: &str, data: HsdMaterial) {
    let mat_map = harness
        .doc
        .get_map("hsd")
        .get_or_create_container("materials", LoroMap::new())
        .expect("materials map")
        .get_or_create_container(id, LoroMap::new())
        .expect("material map entry");
    data.reconcile(&mat_map).expect("reconcile material data");
}

#[test]
fn material_entity_spawns() {
    let mut h = TestHarness::new();
    add_material(&h, "mat-0");
    h.commit_and_update();

    let mut q = h
        .app
        .world_mut()
        .query_filtered::<Entity, (With<HsdChild>, Without<NodeId>)>();
    assert_eq!(
        q.iter(h.app.world()).count(),
        1,
        "one material entity expected"
    );

    assert!(
        h.app
            .world()
            .get::<bevy_hsd::HsdChildren>(h.doc_entity)
            .is_some()
    );
}

#[test]
fn material_removed() {
    let mut h = TestHarness::new();
    add_material(&h, "mat-0");
    h.commit_and_update();

    let mut q = h
        .app
        .world_mut()
        .query_filtered::<Entity, (With<HsdChild>, Without<NodeId>)>();
    assert_eq!(q.iter(h.app.world()).count(), 1);

    h.doc
        .get_map("hsd")
        .get_or_create_container("materials", LoroMap::new())
        .expect("materials map")
        .delete("mat-0")
        .expect("delete material");
    h.commit_and_update();

    let mut q2 = h
        .app
        .world_mut()
        .query_filtered::<Entity, (With<HsdChild>, Without<NodeId>)>();
    assert_eq!(
        q2.iter(h.app.world()).count(),
        0,
        "material entity should be gone"
    );
}

#[test]
fn node_material_ref_set() {
    let mut h = TestHarness::new();
    add_material(&h, "mat-0");

    let nodes = h
        .doc
        .get_map("hsd")
        .get_or_create_container("nodes", LoroTree::new())
        .expect("nodes tree");
    let tid = nodes.create(TreeParentId::Root).expect("create node");
    let meta = nodes.get_meta(tid).expect("node meta");
    HsdNodeData {
        material: Some("mat-0".into()),
        ..Default::default()
    }
    .reconcile(&meta)
    .expect("reconcile node data");
    h.commit_and_update();

    let mut q = h.app.world_mut().query::<(&NodeId, &MaterialRef)>();
    assert!(
        q.iter(h.app.world()).next().is_some(),
        "MaterialRef expected on node"
    );
}

#[test]
fn material_with_base_color() {
    let mut h = TestHarness::new();
    set_material(
        &h,
        "mat-red",
        HsdMaterial {
            base_color: Some(vec![1.0, 0.0, 0.0, 1.0]),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let mut q = h
        .app
        .world_mut()
        .query_filtered::<Entity, (With<HsdChild>, Without<NodeId>)>();
    assert_eq!(q.iter(h.app.world()).count(), 1, "material entity expected");
}
