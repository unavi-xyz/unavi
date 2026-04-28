use bevy::prelude::*;
use bevy_hsd::{
    HsdChild, NodeId,
    hydrate::compile::{material::CompiledMaterial, node::MaterialRef},
};
use hsd::{HsdMaterial, HsdNode};
use loro::{LoroMap, LoroTree, TreeParentId};
use loro_surgeon::Reconcile;

mod common;

use common::TestHarness;

const EPSILON: f32 = 1e-5;

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

fn mat_entity(h: &mut TestHarness) -> Entity {
    h.app
        .world_mut()
        .query_filtered::<Entity, (With<HsdChild>, Without<NodeId>)>()
        .iter(h.app.world())
        .next()
        .expect("material entity")
}

fn get_standard_material(h: &TestHarness, ent: Entity) -> &StandardMaterial {
    let compiled = h
        .app
        .world()
        .get::<CompiledMaterial>(ent)
        .expect("CompiledMaterial");
    h.app
        .world()
        .get_resource::<Assets<StandardMaterial>>()
        .expect("assets resource")
        .get(&compiled.0)
        .expect("StandardMaterial asset")
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
    HsdNode {
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
fn material_base_color_value() {
    let mut h = TestHarness::new();
    set_material(
        &h,
        "mat-0",
        HsdMaterial {
            base_color: Some(vec![1.0, 0.0, 0.0, 1.0]),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let ent = mat_entity(&mut h);
    let mat = get_standard_material(&h, ent);
    let Color::Srgba(c) = mat.base_color else {
        panic!("expected Srgba color");
    };
    assert!((c.red - 1.0).abs() < EPSILON, "red: {}", c.red);
    assert!((c.green - 0.0).abs() < EPSILON, "green: {}", c.green);
    assert!((c.blue - 0.0).abs() < EPSILON, "blue: {}", c.blue);
    assert!((c.alpha - 1.0).abs() < EPSILON, "alpha: {}", c.alpha);
}

#[test]
fn material_metallic_value() {
    let mut h = TestHarness::new();
    set_material(
        &h,
        "mat-0",
        HsdMaterial {
            metallic: Some(0.7),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let ent = mat_entity(&mut h);
    let mat = get_standard_material(&h, ent);
    assert!(
        (mat.metallic - 0.7).abs() < EPSILON,
        "metallic: {}",
        mat.metallic
    );
}

#[test]
fn material_roughness_value() {
    let mut h = TestHarness::new();
    set_material(
        &h,
        "mat-0",
        HsdMaterial {
            roughness: Some(0.2),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let ent = mat_entity(&mut h);
    let mat = get_standard_material(&h, ent);
    assert!(
        (mat.perceptual_roughness - 0.2).abs() < EPSILON,
        "roughness: {}",
        mat.perceptual_roughness
    );
}

#[test]
fn material_double_sided() {
    let mut h = TestHarness::new();
    set_material(
        &h,
        "mat-0",
        HsdMaterial {
            double_sided: Some(true),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let ent = mat_entity(&mut h);
    let mat = get_standard_material(&h, ent);
    assert!(mat.double_sided, "double_sided should be true");
}

#[test]
fn material_unlit() {
    let mut h = TestHarness::new();
    set_material(
        &h,
        "mat-0",
        HsdMaterial {
            unlit: Some(true),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let ent = mat_entity(&mut h);
    let mat = get_standard_material(&h, ent);
    assert!(mat.unlit, "unlit should be true");
}

#[test]
fn material_alpha_mode_blend() {
    let mut h = TestHarness::new();
    set_material(
        &h,
        "mat-0",
        HsdMaterial {
            alpha_mode: Some("blend".into()),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let ent = mat_entity(&mut h);
    let mat = get_standard_material(&h, ent);
    assert_eq!(mat.alpha_mode, AlphaMode::Blend);
}

#[test]
fn material_alpha_mode_mask_with_cutoff() {
    let mut h = TestHarness::new();
    set_material(
        &h,
        "mat-0",
        HsdMaterial {
            alpha_mode: Some("mask".into()),
            alpha_cutoff: Some(0.3),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let ent = mat_entity(&mut h);
    let mat = get_standard_material(&h, ent);
    let AlphaMode::Mask(cutoff) = mat.alpha_mode else {
        panic!("expected AlphaMode::Mask, got {:?}", mat.alpha_mode);
    };
    assert!((cutoff - 0.3).abs() < EPSILON, "alpha_cutoff: {cutoff}");
}

#[test]
fn material_name() {
    let mut h = TestHarness::new();
    set_material(
        &h,
        "mat-0",
        HsdMaterial {
            name: Some("my-mat".into()),
            ..Default::default()
        },
    );
    h.commit_and_update();

    let ent = mat_entity(&mut h);
    let name = h
        .app
        .world()
        .get::<Name>(ent)
        .expect("Name component on material entity");
    assert_eq!(name.as_str(), "my-mat");
}
