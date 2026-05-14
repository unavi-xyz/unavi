use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::{mesh::PrimitiveTopology, prelude::*};
use bevy_hsd::hydrate::compile::mesh::{CompiledMesh, MeshState};
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

fn mesh_entity(h: &mut TestHarness) -> Entity {
    h.app
        .world_mut()
        .query_filtered::<Entity, With<HsdChild>>()
        .iter(h.app.world())
        .next()
        .expect("mesh entity")
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

#[test]
fn mesh_topology_line_list() {
    let mut h = TestHarness::new();
    h.attach_inline_mesh(
        "mesh-0",
        MeshState {
            topology: PrimitiveTopology::LineList,
            positions: Some(vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
            ..Default::default()
        },
    );

    let ent = mesh_entity(&mut h);
    let compiled = h
        .app
        .world()
        .get::<CompiledMesh>(ent)
        .expect("CompiledMesh");
    let assets = h
        .app
        .world()
        .get_resource::<Assets<Mesh>>()
        .expect("Mesh assets");
    let mesh = assets.get(&compiled.0).expect("Mesh asset");
    assert_eq!(mesh.primitive_topology(), PrimitiveTopology::LineList);
}

#[test]
#[expect(clippy::float_cmp)]
fn mesh_attribute_positions() {
    let mut h = TestHarness::new();
    h.attach_inline_mesh(
        "mesh-0",
        MeshState {
            positions: Some(vec![
                0.0, 0.0, 0.0, // v0
                1.0, 0.0, 0.0, // v1
                0.0, 1.0, 0.0, // v2
            ]),
            ..Default::default()
        },
    );

    let ent = mesh_entity(&mut h);
    let compiled = h
        .app
        .world()
        .get::<CompiledMesh>(ent)
        .expect("CompiledMesh");
    let assets = h
        .app
        .world()
        .get_resource::<Assets<Mesh>>()
        .expect("Mesh assets");
    let mesh = assets.get(&compiled.0).expect("Mesh asset");

    let Some(VertexAttributeValues::Float32x3(pos)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("POSITION attribute missing or wrong type");
    };
    assert_eq!(pos.len(), 3, "three vertices expected");
    assert_eq!(pos[0], [0.0, 0.0, 0.0]);
    assert_eq!(pos[1], [1.0, 0.0, 0.0]);
    assert_eq!(pos[2], [0.0, 1.0, 0.0]);
}

#[test]
#[expect(clippy::float_cmp)]
fn mesh_attribute_uv0() {
    let mut h = TestHarness::new();
    h.attach_inline_mesh(
        "mesh-0",
        MeshState {
            uv0: Some(vec![
                0.0, 0.0, // 0
                1.0, 0.0, // 1
            ]),
            ..Default::default()
        },
    );

    let ent = mesh_entity(&mut h);
    let compiled = h
        .app
        .world()
        .get::<CompiledMesh>(ent)
        .expect("CompiledMesh");
    let assets = h
        .app
        .world()
        .get_resource::<Assets<Mesh>>()
        .expect("Mesh assets");
    let mesh = assets.get(&compiled.0).expect("Mesh asset");

    let Some(VertexAttributeValues::Float32x2(pos)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) else {
        panic!("UV_0 attribute missing or wrong type");
    };
    assert_eq!(pos.len(), 2, "two items expected");
    assert_eq!(pos[0], [0.0, 0.0]);
    assert_eq!(pos[1], [1.0, 0.0]);
}

// TODO test all mesh attributes

#[test]
fn mesh_indices() {
    let mut h = TestHarness::new();
    h.attach_inline_mesh(
        "mesh-0",
        MeshState {
            positions: Some(vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]),
            indices: Some(vec![0, 1, 2]),
            ..Default::default()
        },
    );

    let ent = mesh_entity(&mut h);
    let compiled = h
        .app
        .world()
        .get::<CompiledMesh>(ent)
        .expect("CompiledMesh");
    let assets = h
        .app
        .world()
        .get_resource::<Assets<Mesh>>()
        .expect("Mesh assets");
    let mesh = assets.get(&compiled.0).expect("Mesh asset");

    let Some(Indices::U32(idx)) = mesh.indices() else {
        panic!("U32 indices expected");
    };
    assert_eq!(idx, &[0, 1, 2]);
}
