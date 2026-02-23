use std::sync::Arc;

use avian3d::{PhysicsPlugins, prelude::PhysicsDebugPlugin};
use bevy::{
    mesh::Indices,
    pbr::{Atmosphere, AtmosphereSettings},
    prelude::*,
};
use bevy_hsd::HsdPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_wds::{LocalBlobs, WdsPlugin};
use bytemuck::cast_slice;
use iroh_blobs::store::mem::MemStore;
use loro::LoroDoc;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PanOrbitCameraPlugin,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin,
            HsdPlugin,
            WdsPlugin,
        ))
        .add_systems(Startup, (setup_scene, load_hsd))
        .run();
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(4.0, 5.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
        Atmosphere::EARTH,
        AtmosphereSettings::default(),
    ));

    commands.spawn((
        Transform::from_xyz(-2.0, 4.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        DirectionalLight::default(),
    ));
}

#[derive(Component)]
#[expect(dead_code)]
struct BlobStore(MemStore);

fn load_hsd(mut commands: Commands) {
    let (tx, rx) = std::sync::mpsc::channel();

    unavi_wasm_compat::spawn_thread(async move {
        let store = MemStore::default();
        tx.send(store.clone()).expect("send");
        tokio::signal::ctrl_c().await.expect("signal");
    });

    let store = rx.recv().expect("recv");
    commands.spawn(LocalBlobs(store.blobs().clone()));

    let doc = LoroDoc::new();
    let hsd = doc.get_map("hsd");

    // Add a material.
    let materials = hsd
        .get_or_create_container("materials", loro::LoroList::new())
        .expect("materials list");
    let mat_map = materials
        .push_container(loro::LoroMap::new())
        .expect("push mat");
    mat_map
        .insert_container("base_color", {
            let list = loro::LoroList::new();
            list.push(0.4).expect("push");
            list.push(0.3).expect("push");
            list.push(0.7).expect("push");
            list
        })
        .expect("base_color");
    mat_map.insert("metallic", 0.8).expect("metallic");
    mat_map.insert("roughness", 0.4).expect("roughness");

    // Add a mesh.
    let x_length = 2.0;
    let y_length = 0.5;
    let z_length = 1.0;
    let cube = Cuboid::new(x_length, y_length, z_length).mesh().build();

    let meshes = hsd
        .get_or_create_container("meshes", loro::LoroList::new())
        .expect("meshes list");
    let mesh_map = meshes
        .push_container(loro::LoroMap::new())
        .expect("push mesh");
    mesh_map.insert("topology", 3i64).expect("topology");

    // Attributes map.
    let attrs = mesh_map
        .get_or_create_container("attributes", loro::LoroMap::new())
        .expect("attrs");

    let Some(Indices::U32(indices)) = cube.indices() else {
        unreachable!()
    };
    let indices_hash = add_blob(&store, cast_slice(indices));
    mesh_map
        .insert("indices", indices_hash.as_bytes().to_vec())
        .expect("indices");

    let Some(points) = cube.attribute(Mesh::ATTRIBUTE_POSITION) else {
        unreachable!()
    };
    let hash = add_blob(&store, points.get_bytes());
    attrs
        .insert("POSITION", hash.as_bytes().to_vec())
        .expect("position");

    let Some(normals) = cube.attribute(Mesh::ATTRIBUTE_NORMAL) else {
        unreachable!()
    };
    let hash = add_blob(&store, normals.get_bytes());
    attrs
        .insert("NORMAL", hash.as_bytes().to_vec())
        .expect("normal");

    // Add a node.
    let nodes = hsd
        .get_or_create_container("nodes", loro::LoroTree::new())
        .expect("nodes tree");
    let node_id = nodes.create(None).expect("create node");
    let meta = nodes.get_meta(node_id).expect("meta");
    meta.insert("mesh", 0).expect("mesh ref");
    meta.insert("material", 0).expect("material ref");
    meta.insert_container("translation", {
        let list = loro::LoroList::new();
        list.push(0.0).expect("push");
        list.push(-0.5).expect("push");
        list.push(0.0).expect("push");
        list
    })
    .expect("translation");

    // Collider.
    let collider = meta
        .get_or_create_container("collider", loro::LoroMap::new())
        .expect("collider");
    collider.insert("shape", "cuboid").expect("shape");
    collider
        .insert_container("size", {
            let list = loro::LoroList::new();
            list.push(f64::from(x_length)).expect("push");
            list.push(f64::from(y_length)).expect("push");
            list.push(f64::from(z_length)).expect("push");
            list
        })
        .expect("size");

    commands.spawn(bevy_hsd::HsdDoc(Arc::new(doc)));

    // Keep store alive in memory.
    commands.spawn(BlobStore(store));
}

fn add_blob(store: &MemStore, bytes: &[u8]) -> blake3::Hash {
    let hash = blake3::hash(bytes);
    let store = store.clone();
    let bytes = bytes.to_vec();
    unavi_wasm_compat::spawn_thread(async move {
        store.add_bytes(bytes).await.expect("add bytes");
    });
    hash
}
