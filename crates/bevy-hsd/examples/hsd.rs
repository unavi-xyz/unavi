use bevy::{
    mesh::{
        Indices,
        VertexAttributeValues,
    },
    prelude::*,
};
use bevy_async::task;
use bevy_hsd::{
    HsdPlugin,
    document::Hsd,
};
use bevy_iroh::store::LocalBlobs;
use bevy_panorbit_camera::{
    PanOrbitCamera,
    PanOrbitCameraPlugin,
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
            self,
            MeshIndices,
            MeshStream,
            Topology,
        },
        shader::{
            ShaderGraph,
            graph::{
                DisplacementGraph,
                LitOutput,
                SurfaceGraph,
                SurfaceOutput,
                UnlitOutput,
            },
            node::{
                Node,
                Port,
            },
            value::GraphValue,
        },
        xform::XformAttr,
    },
    id::PrimId,
    state::HsdState,
};
use iroh_blobs::{
    api::blobs::Blobs,
    store::mem::MemStore,
};

const CUBE_SIZE: f32 = 1.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PanOrbitCameraPlugin,
            bevy_iroh::IrohPlugin,
            HsdPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.08, 0.09, 0.13)))
        .add_systems(Startup, (setup_scene, load_hsd))
        .run();
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 4.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
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
#[expect(dead_code, reason = "held so the store outlives the blobs handle")]
struct BlobStore(MemStore);

fn load_hsd(mut commands: Commands) {
    let (store, blobs) = spawn_mem_store();
    commands.spawn(LocalBlobs(blobs));

    let mut state = HsdState::new();
    populate(&mut state);

    commands.spawn(Hsd::new(state));
    commands.spawn(BlobStore(store));
}

fn populate(state: &mut HsdState) {
    let red = material_prim(state, ColorVec(vec![0.9, 0.2, 0.15, 1.0]), 0.1, 0.35);
    let blue = material_prim(state, ColorVec(vec![0.15, 0.4, 0.95, 1.0]), 0.8, 0.2);

    let cube = cube_mesh();

    for (offset, target) in [
        (Vec3::new(-2.0, 0.0, 0.0), red),
        (Vec3::new(0.0, 0.0, 0.0), blue),
        (Vec3::new(2.0, 0.0, 0.0), red),
        (Vec3::new(-1.0, 0.0, -2.0), blue),
        (Vec3::new(1.0, 0.0, -2.0), red),
    ] {
        let prim = state.create_prim(None);
        write_mesh(state, prim, &cube);
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
            .set_relationship(prim, &material::BINDING, target)
            .expect("binding");
    }

    // Two effects a fixed `MaterialAttr` cannot express, on smooth spheres: a
    // cube's per-face normals split apart when displaced along them.
    let sphere = sphere_mesh();

    shader_graph_cube(state, &sphere, Vec3::new(-1.0, 0.0, 2.0), glow_graph());
    shader_graph_cube(state, &sphere, Vec3::new(1.0, 0.0, 2.0), pulse_graph());
}

fn material_prim(
    state: &mut HsdState,
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

fn shader_graph_cube(
    state: &mut HsdState,
    mesh: &Mesh,
    offset: Vec3,
    graph: ShaderGraph,
) -> PrimId {
    let prim = state.create_prim(None);
    write_mesh(state, prim, mesh);
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
    state.set_attribute(prim, &graph).expect("shader graph");

    prim
}

/// Unlit fresnel rim, the rim tint a public input so it can be overridden.
fn glow_graph() -> ShaderGraph {
    ShaderGraph {
        public_inputs: vec![GraphValue::Color([0.2, 0.8, 1.0, 1.0])],
        surface:       SurfaceGraph {
            nodes: vec![
                Node::Fresnel {
                    power: Port::Const(GraphValue::Float(2.5)),
                },
                Node::Lerp {
                    a: Port::Const(GraphValue::Color([0.02, 0.02, 0.05, 1.0])),
                    b: Port::Input(0),
                    t: Port::Node(0),
                },
            ],
            output: SurfaceOutput::Unlit(UnlitOutput {
                color:                Port::Node(1),
                alpha_clip_threshold: None,
            }),
            ..Default::default()
        },
        displacement:  None,
    }
}

/// Lit PBR with a slow sine displacement that breathes the sphere along its
/// normals.
fn pulse_graph() -> ShaderGraph {
    ShaderGraph {
        public_inputs: Vec::new(),
        surface:       SurfaceGraph {
            nodes: Vec::new(),
            output: SurfaceOutput::Lit(LitOutput {
                base_color: Some(Port::Const(GraphValue::Color([0.9, 0.5, 0.1, 1.0]))),
                metallic: Some(Port::Const(GraphValue::Float(0.1))),
                roughness: Some(Port::Const(GraphValue::Float(0.4))),
                ..Default::default()
            }),
            ..Default::default()
        },
        displacement:  Some(DisplacementGraph {
            nodes:                 vec![
                Node::Time,
                Node::Sin { x: Port::Node(0) },
                Node::LocalNormal,
                Node::Mul {
                    a: Port::Node(2),
                    b: Port::Node(1),
                },
                Node::Mul {
                    a: Port::Node(3),
                    b: Port::Const(GraphValue::Float(0.15)),
                },
            ],
            position_offset:       Some(Port::Node(4)),
            normal_override:       None,
            world_position_offset: None,
        }),
    }
}

fn cube_mesh() -> Mesh {
    Cuboid::new(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE).mesh().build()
}

/// Smoothly-normalled sphere, so a displacement graph breathes the whole
/// shell rather than splitting per-face vertices apart.
fn sphere_mesh() -> Mesh {
    Sphere::new(CUBE_SIZE / 2.0).mesh().build()
}

/// Writes a Bevy mesh's buffers as `mesh/topology`, `mesh/indices` and one
/// `mesh/stream:<NAME>` field per vertex attribute the mesh carries.
fn write_mesh(state: &mut HsdState, prim: PrimId, mesh: &Mesh) {
    state
        .set_attribute(prim, &Topology::TriangleList)
        .expect("topology");

    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        write_stream(state, prim, "POSITION", cast_slice(positions).to_vec());
    }
    if let Some(VertexAttributeValues::Float32x3(normals)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
    {
        write_stream(state, prim, "NORMAL", cast_slice(normals).to_vec());
    }
    if let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        write_stream(state, prim, "UV_0", cast_slice(uvs).to_vec());
    }
    if let Some(Indices::U32(idx)) = mesh.indices() {
        state
            .set_attribute(prim, &MeshIndices(cast_slice(idx).to_vec()))
            .expect("indices");
    }
}

fn write_stream(state: &mut HsdState, prim: PrimId, name: &str, bytes: Vec<u8>) {
    let field = mesh::stream(name).expect("valid stream name");
    state
        .set_payload(prim, &field, &MeshStream(bytes))
        .expect("stream");
}

fn spawn_mem_store() -> (MemStore, Blobs) {
    let (tx, rx) = async_channel::bounded(1);
    task::spawn(async move {
        let store = MemStore::default();
        let blobs = store.blobs().clone();
        tx.send((store, blobs)).await.expect("send");
        std::future::pending::<()>().await;
    });
    rx.recv_blocking().expect("recv store")
}
