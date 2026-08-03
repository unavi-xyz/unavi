use std::{
    collections::BTreeMap,
    sync::Arc,
};

use bevy::{
    light::{
        Atmosphere,
        atmosphere::ScatteringMedium,
    },
    mesh::{
        Indices,
        VertexAttributeValues,
    },
    pbr::AtmosphereSettings,
    prelude::*,
};
use bevy_hsd::{
    Hsd,
    HsdPlugin,
};
use bevy_panorbit_camera::{
    PanOrbitCamera,
    PanOrbitCameraPlugin,
};
use bevy_wds::{
    LocalBlobs,
    WdsPlugin,
};
use bytemuck::cast_slice;
use hsd::{
    HSD_CONTAINER_ID,
    PrimMeta,
    attributes::{
        Attributes,
        material::{
            ColorVec,
            MaterialAttr,
        },
        mesh::{
            MeshAttr,
            Topology,
        },
        xform::XformAttr,
    },
};
use iroh_blobs::{
    api::blobs::Blobs,
    store::mem::MemStore,
};
use loro::LoroDoc;
use loro_surgeon::{
    Reconcile,
    bytes::ByteArray,
    reconcile::RootReconciler,
};
use unavi_util::async_task::spawn_async_task;

const CUBE_SIZE: f32 = 1.0;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin, WdsPlugin, HsdPlugin))
        .add_systems(Startup, (setup_scene, load_hsd))
        .run();
}

fn setup_scene(mut commands: Commands, mut scattering_mediums: ResMut<Assets<ScatteringMedium>>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 4.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
        Atmosphere::earth(scattering_mediums.add(ScatteringMedium::default())),
        AtmosphereSettings::default(),
    ));

    commands.spawn((
        Transform::from_xyz(-2.0, 6.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
    ));
}

#[derive(Component)]
#[expect(dead_code)]
struct BlobStore(MemStore);

fn load_hsd(mut commands: Commands) {
    let (store, blobs) = spawn_mem_store();
    commands.spawn(LocalBlobs(blobs.clone()));

    let doc = Arc::new(LoroDoc::new());
    populate(&doc, &blobs);

    commands.spawn(Hsd(Arc::clone(&doc)));
    commands.spawn(BlobStore(store));
}

fn populate(doc: &LoroDoc, blobs: &Blobs) {
    let tree = doc.get_tree(&*HSD_CONTAINER_ID);

    let red = tree.create(None).expect("create red");
    reconcile_prim(
        &tree.get_meta(red).expect("red meta"),
        Attributes {
            material: Some(MaterialAttr {
                base_color: Some(ColorVec(vec![0.9, 0.2, 0.15, 1.0])),
                metallic: Some(0.1),
                roughness: Some(0.35),
                ..Default::default()
            }),
            ..Default::default()
        },
        None,
    );

    let blue = tree.create(None).expect("create blue");
    reconcile_prim(
        &tree.get_meta(blue).expect("blue meta"),
        Attributes {
            material: Some(MaterialAttr {
                base_color: Some(ColorVec(vec![0.15, 0.4, 0.95, 1.0])),
                metallic: Some(0.8),
                roughness: Some(0.2),
                ..Default::default()
            }),
            ..Default::default()
        },
        None,
    );

    let mesh_attr = build_cube_mesh_attr(blobs);

    for (offset, target) in [
        (Vec3::new(-2.0, 0.0, 0.0), red),
        (Vec3::new(0.0, 0.0, 0.0), blue),
        (Vec3::new(2.0, 0.0, 0.0), red),
        (Vec3::new(-1.0, 0.0, -2.0), blue),
        (Vec3::new(1.0, 0.0, -2.0), red),
    ] {
        let prim = tree.create(None).expect("create cube");
        reconcile_prim(
            &tree.get_meta(prim).expect("cube meta"),
            Attributes {
                mesh: Some(mesh_attr.clone()),
                xform: Some(XformAttr {
                    rotation:    [0.0, 0.0, 0.0, 1.0],
                    scale:       [1.0, 1.0, 1.0],
                    translation: offset.to_array(),
                }),
                ..Default::default()
            },
            Some(BTreeMap::from([(
                "material".to_string(),
                target.to_string(),
            )])),
        );
    }

    doc.commit();
}

fn build_cube_mesh_attr(blobs: &Blobs) -> MeshAttr {
    let cube = Cuboid::new(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE).mesh().build();

    let mut attrs = BTreeMap::new();
    let mut indices = None;

    if let Some(VertexAttributeValues::Float32x3(positions)) =
        cube.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        attrs.insert("POSITION".to_string(), upload(blobs, cast_slice(positions)));
    }
    if let Some(VertexAttributeValues::Float32x3(normals)) = cube.attribute(Mesh::ATTRIBUTE_NORMAL)
    {
        attrs.insert("NORMAL".to_string(), upload(blobs, cast_slice(normals)));
    }
    if let Some(VertexAttributeValues::Float32x2(uvs)) = cube.attribute(Mesh::ATTRIBUTE_UV_0) {
        attrs.insert("UV_0".to_string(), upload(blobs, cast_slice(uvs)));
    }
    if let Some(Indices::U32(idx)) = cube.indices() {
        indices = Some(upload(blobs, cast_slice(idx)));
    }

    MeshAttr {
        attributes: attrs,
        indices,
        topology: Topology::TriangleList,
    }
}

fn upload(blobs: &Blobs, bytes: &[u8]) -> ByteArray<32> {
    let hash = blake3::hash(bytes);
    let blobs = blobs.clone();
    let payload = bytes.to_vec();
    spawn_async_task(async move {
        blobs.add_slice(&payload).await.expect("add slice");
    });
    ByteArray::<32>::new(*hash.as_bytes())
}

fn reconcile_prim(
    meta: &loro::LoroMap,
    attributes: Attributes,
    relationships: Option<BTreeMap<String, String>>,
) {
    let prim = PrimMeta {
        attributes: Some(attributes),
        relationships,
    };
    prim.reconcile(RootReconciler::new(meta.clone()))
        .expect("reconcile prim");
}

fn spawn_mem_store() -> (MemStore, Blobs) {
    let (tx, rx) = async_channel::bounded(1);
    spawn_async_task(async move {
        let store = MemStore::default();
        let blobs = store.blobs().clone();
        tx.send((store, blobs)).await.expect("send");
        std::future::pending::<()>().await;
    });
    rx.recv_blocking().expect("recv store")
}
