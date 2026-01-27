use bevy::{
    mesh::Indices,
    pbr::{Atmosphere, AtmosphereSettings},
    prelude::*,
};
use bevy_hsd::{
    HsdPlugin, Stage,
    stage::{LayerData, OpinionData, StageData},
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_wds::{LocalBlobs, WdsPlugin};
use bytemuck::cast_slice;
use iroh_blobs::store::mem::MemStore;
use loro::{LoroBinaryValue, LoroListValue, LoroMapValue, LoroValue};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin, HsdPlugin, WdsPlugin))
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

    let mut attrs = LoroMapValue::default();

    let mut xform_pos = LoroListValue::default();
    *xform_pos.make_mut() = vec![
        LoroValue::Double(0.0),
        LoroValue::Double(-0.5),
        LoroValue::Double(0.0),
    ];
    attrs
        .make_mut()
        .insert("xform/pos".to_string(), LoroValue::List(xform_pos));

    attrs.make_mut().insert(
        "mesh/topology".to_string(),
        LoroValue::I64(
            3, // TriangleList
        ),
    );

    let cube = Cuboid::new(2.0, 0.5, 1.0).mesh().build();

    let Some(Indices::U32(indices)) = cube.indices() else {
        unreachable!()
    };
    let indices_bytes = cast_slice(indices);
    add_blob_ref(&store, &mut attrs, "mesh/indices", indices_bytes);

    let Some(points) = cube.attribute(Mesh::ATTRIBUTE_POSITION) else {
        unreachable!()
    };
    add_blob_ref(&store, &mut attrs, "mesh/points", points.get_bytes());

    let Some(normals) = cube.attribute(Mesh::ATTRIBUTE_NORMAL) else {
        unreachable!()
    };
    add_blob_ref(&store, &mut attrs, "mesh/normals", normals.get_bytes());

    let stage = StageData {
        layers: vec![LayerData {
            enabled: true,
            opinions: vec![OpinionData { node: 0, attrs }],
        }],
    };

    commands.spawn(Stage(stage));

    // Keep store alive in memory.
    commands.spawn(BlobStore(store));
}

fn add_blob_ref(store: &MemStore, attrs: &mut LoroMapValue, key: impl Into<String>, bytes: &[u8]) {
    let hash = blake3::hash(bytes);

    let store = store.clone();
    let bytes = bytes.to_vec();
    unavi_wasm_compat::spawn_thread(async move {
        store.add_bytes(bytes).await.expect("add bytes");
    });

    let mut bin_value = LoroBinaryValue::default();
    *bin_value.make_mut() = hash.as_bytes().to_vec();
    attrs
        .make_mut()
        .insert(key.into(), LoroValue::Binary(bin_value));
}
