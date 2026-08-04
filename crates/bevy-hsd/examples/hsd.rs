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
    attributes::{
        material::{
            self,
            ColorVec,
            MaterialAttr,
        },
        mesh::{
            MeshAttr,
            Topology,
        },
        slots,
        xform::XformAttr,
    },
    id::{
        BlobId,
        PrimId,
    },
    state::{
        SceneState,
        entry::BulkRef,
    },
};
use iroh_blobs::{
    api::blobs::Blobs,
    store::mem::MemStore,
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

    let mut state = SceneState::new();
    populate(&mut state, &blobs);

    commands.spawn(Hsd::new(state));
    commands.spawn(BlobStore(store));
}

fn populate(state: &mut SceneState, blobs: &Blobs) {
    let red = material_prim(state, ColorVec(vec![0.9, 0.2, 0.15, 1.0]), 0.1, 0.35);
    let blue = material_prim(state, ColorVec(vec![0.15, 0.4, 0.95, 1.0]), 0.8, 0.2);

    let buffers = cube_buffers(blobs);

    for (offset, target) in [
        (Vec3::new(-2.0, 0.0, 0.0), red),
        (Vec3::new(0.0, 0.0, 0.0), blue),
        (Vec3::new(2.0, 0.0, 0.0), red),
        (Vec3::new(-1.0, 0.0, -2.0), blue),
        (Vec3::new(1.0, 0.0, -2.0), red),
    ] {
        let prim = state.create_prim(None);
        state
            .set_attribute(
                prim,
                &MeshAttr {
                    topology: Topology::TriangleList,
                },
            )
            .expect("mesh");
        state
            .set_attribute(
                prim,
                &XformAttr {
                    rotation:    [0.0, 0.0, 0.0, 1.0],
                    scale:       [1.0, 1.0, 1.0],
                    translation: offset.to_array(),
                },
            )
            .expect("xform");
        state
            .set_relationship(prim, material::BINDING, target)
            .expect("binding");

        for (slot, value) in &buffers {
            state.set_bulk(prim, slot, *value).expect("bulk");
        }
    }
}

fn material_prim(
    state: &mut SceneState,
    base_color: ColorVec,
    metallic: f64,
    roughness: f64,
) -> PrimId {
    let prim = state.create_prim(None);
    state
        .set_attribute(
            prim,
            &MaterialAttr {
                base_color: Some(base_color),
                metallic: Some(metallic),
                roughness: Some(roughness),
                ..Default::default()
            },
        )
        .expect("material");
    prim
}

fn cube_buffers(blobs: &Blobs) -> Vec<(String, BulkRef)> {
    let cube = Cuboid::new(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE).mesh().build();

    let mut out = Vec::new();

    if let Some(VertexAttributeValues::Float32x3(positions)) =
        cube.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        out.push((
            slots::mesh_attribute("POSITION"),
            upload(blobs, cast_slice(positions)),
        ));
    }
    if let Some(VertexAttributeValues::Float32x3(normals)) = cube.attribute(Mesh::ATTRIBUTE_NORMAL)
    {
        out.push((
            slots::mesh_attribute("NORMAL"),
            upload(blobs, cast_slice(normals)),
        ));
    }
    if let Some(VertexAttributeValues::Float32x2(uvs)) = cube.attribute(Mesh::ATTRIBUTE_UV_0) {
        out.push((
            slots::mesh_attribute("UV_0"),
            upload(blobs, cast_slice(uvs)),
        ));
    }
    if let Some(Indices::U32(idx)) = cube.indices() {
        out.push((
            slots::MESH_INDICES.to_owned(),
            upload(blobs, cast_slice(idx)),
        ));
    }

    out
}

fn upload(blobs: &Blobs, bytes: &[u8]) -> BulkRef {
    let hash = blake3::hash(bytes);
    let size = bytes.len() as u64;
    let blobs = blobs.clone();
    let payload = bytes.to_vec();
    spawn_async_task(async move {
        blobs.add_slice(&payload).await.expect("add slice");
    });
    BulkRef {
        hash: BlobId(*hash.as_bytes()),
        size,
    }
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
