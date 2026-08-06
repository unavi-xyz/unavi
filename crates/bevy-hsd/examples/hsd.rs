use bevy::{
    mesh::{
        Indices,
        VertexAttributeValues,
    },
    prelude::*,
};
use bevy_hsd::{
    Hsd,
    HsdPlugin,
    attributes::material_graph::ShaderGraphMaterial,
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
        material_graph::{
            DisplacementGraph,
            GraphValue,
            LitOutput,
            Node,
            NodeKind,
            Port,
            ShaderGraph,
            SurfaceGraph,
            SurfaceOutput,
            UnlitOutput,
        },
        mesh::{
            MeshAttr,
            Topology,
        },
        slots,
        xform::XformAttr,
    },
    id::PrimId,
    state::SceneState,
};
use iroh_blobs::{
    api::blobs::Blobs,
    store::mem::MemStore,
};
use unavi_util::async_task::spawn_async_task;

const CUBE_SIZE: f32 = 1.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PanOrbitCameraPlugin,
            WdsPlugin,
            HsdPlugin,
            MaterialPlugin::<ShaderGraphMaterial>::default(),
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
#[expect(dead_code)]
struct BlobStore(MemStore);

fn load_hsd(mut commands: Commands) {
    let (store, blobs) = spawn_mem_store();
    commands.spawn(LocalBlobs(blobs));

    let mut state = SceneState::new();
    populate(&mut state);

    commands.spawn(Hsd::new(state));
    commands.spawn(BlobStore(store));
}

fn populate(state: &mut SceneState) {
    let red = material_prim(state, ColorVec(vec![0.9, 0.2, 0.15, 1.0]), 0.1, 0.35);
    let blue = material_prim(state, ColorVec(vec![0.15, 0.4, 0.95, 1.0]), 0.8, 0.2);

    let buffers = cube_buffers();

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
            state.set_slot(prim, slot, value.clone()).expect("slot");
        }
    }

    // Two effects a fixed `MaterialAttr` cannot express: an unlit fresnel
    // glow and a sine-driven vertex displacement. Both use a smooth sphere —
    // a cube's per-face normals split apart when displaced along them.
    let graph_buffers = sphere_buffers();

    shader_graph_cube(
        state,
        &graph_buffers,
        Vec3::new(-1.0, 0.0, 2.0),
        glow_graph(),
    );
    shader_graph_cube(
        state,
        &graph_buffers,
        Vec3::new(1.0, 0.0, 2.0),
        pulse_graph(),
    );
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

/// A prim carrying a compiled shader graph as a `material:graph_data` slot
/// rather than a bound PBR `MaterialAttr` — a compiled graph is exactly the
/// kind of content that belongs inline in a slot.
fn shader_graph_cube(
    state: &mut SceneState,
    buffers: &[(String, Vec<u8>)],
    offset: Vec3,
    graph: ShaderGraph,
) -> PrimId {
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
    for (slot, value) in buffers {
        state.set_slot(prim, slot, value.clone()).expect("slot");
    }

    let bytes = graph.encode().expect("encode shader graph");
    state
        .set_slot(prim, slots::MATERIAL_GRAPH_DATA, bytes)
        .expect("shader graph slot");

    prim
}

/// Unlit fresnel rim, the rim tint a public input so it can be overridden.
fn glow_graph() -> ShaderGraph {
    ShaderGraph {
        public_inputs: vec![GraphValue::Color([0.2, 0.8, 1.0, 1.0])],
        surface:       SurfaceGraph {
            nodes:  vec![
                Node {
                    kind: NodeKind::Fresnel {
                        power: Port::Const(GraphValue::Float(2.5)),
                    },
                },
                Node {
                    kind: NodeKind::Lerp {
                        a: Port::Const(GraphValue::Color([0.02, 0.02, 0.05, 1.0])),
                        b: Port::Input(0),
                        t: Port::Node(0),
                    },
                },
            ],
            output: SurfaceOutput::Unlit(UnlitOutput {
                color:                Port::Node(1),
                alpha_clip_threshold: None,
            }),
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
            nodes:  Vec::new(),
            output: SurfaceOutput::Lit(LitOutput {
                base_color: Some(Port::Const(GraphValue::Color([0.9, 0.5, 0.1, 1.0]))),
                metallic: Some(Port::Const(GraphValue::Float(0.1))),
                roughness: Some(Port::Const(GraphValue::Float(0.4))),
                ..Default::default()
            }),
        },
        displacement:  Some(DisplacementGraph {
            nodes:           vec![
                Node {
                    kind: NodeKind::Time,
                },
                Node {
                    kind: NodeKind::Sin { x: Port::Node(0) },
                },
                // `Mul` needs matching kinds, so scale the scalar first, then
                // drive `Lerp`'s `t` with it: `mix(0, normal, sin*0.15)`.
                Node {
                    kind: NodeKind::Mul {
                        a: Port::Node(1),
                        b: Port::Const(GraphValue::Float(0.15)),
                    },
                },
                Node {
                    kind: NodeKind::LocalNormal,
                },
                Node {
                    kind: NodeKind::Lerp {
                        a: Port::Const(GraphValue::Vec3([0.0, 0.0, 0.0])),
                        b: Port::Node(3),
                        t: Port::Node(2),
                    },
                },
            ],
            position_offset: Some(Port::Node(4)),
            normal_override: None,
        }),
    }
}

fn cube_buffers() -> Vec<(String, Vec<u8>)> {
    let cube = Cuboid::new(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE).mesh().build();
    mesh_buffers(cube)
}

/// Smoothly-normalled sphere, so a displacement graph breathes the whole
/// shell rather than splitting a cube's per-face vertices apart.
fn sphere_buffers() -> Vec<(String, Vec<u8>)> {
    mesh_buffers(Sphere::new(CUBE_SIZE / 2.0).mesh().build())
}

fn mesh_buffers(mesh: Mesh) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();

    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        out.push((
            slots::mesh_attribute("POSITION"),
            cast_slice(positions).to_vec(),
        ));
    }
    if let Some(VertexAttributeValues::Float32x3(normals)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
    {
        out.push((
            slots::mesh_attribute("NORMAL"),
            cast_slice(normals).to_vec(),
        ));
    }
    if let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        out.push((slots::mesh_attribute("UV_0"), cast_slice(uvs).to_vec()));
    }
    if let Some(Indices::U32(idx)) = mesh.indices() {
        out.push((slots::MESH_INDICES.to_owned(), cast_slice(idx).to_vec()));
    }

    out
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
