use std::collections::BTreeMap;

use bevy::{
    pbr::MeshMaterial3d,
    prelude::*,
    render::render_resource::Face,
};
use bevy_hsd::attributes::material_graph::{
    HsdMaterialGraphSlot,
    HsdShaderGraphMaterial,
    ShaderGraphMaterial,
    ShaderGraphOverridesData,
};
use hsd::attributes::{
    material,
    material_graph::{
        ShaderGraph,
        graph::{
            BlendMode,
            CullMode,
            DisplacementGraph,
            SurfaceGraph,
            SurfaceOutput,
            UnlitOutput,
        },
        node::{
            Node,
            Port,
        },
        overrides::GraphOverridesAttr,
        value::GraphValue,
    },
    slots,
};
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

/// Unlit rim glow: no lighting pass, so no `PbrInput` at all — the shape a
/// beam/hologram/sky effect needs, unreachable through a fixed PBR terminal
/// set.
fn glow_graph() -> ShaderGraph {
    ShaderGraph {
        public_inputs: vec![GraphValue::Color([0.1, 0.6, 1.0, 1.0])],
        surface:       SurfaceGraph {
            nodes: vec![
                Node::Fresnel {
                    power: Port::Const(GraphValue::Float(2.0)),
                },
                Node::Lerp {
                    a: Port::Const(GraphValue::Color([0.0, 0.0, 0.0, 1.0])),
                    b: Port::Input(0),
                    t: Port::Node(0),
                },
            ],
            output: SurfaceOutput::Unlit(UnlitOutput {
                color:                Port::Node(1),
                alpha_clip_threshold: None,
            }),
            blend: BlendMode::Add,
            ..Default::default()
        },
        displacement:  None,
    }
}

/// No `GraphOverridesAttr` set: `HsdSlots` alone must trigger the pipeline,
/// since that attribute is optional and this is the common case.
#[traced_test]
#[rstest]
fn test_shader_graph_without_overrides(#[from(ctx_wds)] mut ctx: TestContext) {
    let bytes = glow_graph().encode().expect("encode graph");

    let prim = ctx.create_prim();
    ctx.set_slot(prim, slots::MATERIAL_GRAPH_DATA, bytes);

    let mut handle: Option<Handle<ShaderGraphMaterial>> = None;
    ctx.tick_until(|world| {
        let mut q = world.query::<(
            &HsdShaderGraphMaterial,
            &MeshMaterial3d<ShaderGraphMaterial>,
        )>();
        let Some((hsd_mat, mesh_mat)) = q.iter(world).next() else {
            return false;
        };
        assert_eq!(hsd_mat.0, mesh_mat.0);
        handle = Some(hsd_mat.0.clone());
        true
    });

    let handle = handle.expect("shader graph material handle");
    let assets = ctx.app.world().resource::<Assets<ShaderGraphMaterial>>();
    let material = assets.get(&handle).expect("material asset");

    // Public input 0 (the rim tint) keeps the graph's own default: no
    // overrides attribute was ever set on this prim.
    assert_eq!(material.params.inputs[0], Vec4::new(0.1, 0.6, 1.0, 1.0));
    assert_eq!(material.alpha_mode, AlphaMode::Add);
    assert!(material.vertex_shader.is_none(), "no displacement network");
}

/// Overriding public input 0 changes the built material's uniform without
/// touching the compiled graph bytes.
#[traced_test]
#[rstest]
fn test_shader_graph_with_overrides(#[from(ctx_wds)] mut ctx: TestContext) {
    let bytes = glow_graph().encode().expect("encode graph");

    let prim = ctx.create_prim();
    ctx.set_slot(prim, slots::MATERIAL_GRAPH_DATA, bytes);
    ctx.set_attr(
        prim,
        &GraphOverridesAttr {
            overrides: BTreeMap::from([(0, GraphValue::Color([1.0, 0.0, 0.0, 1.0]))]),
        },
    );

    let mut handle: Option<Handle<ShaderGraphMaterial>> = None;
    ctx.tick_until(|world| {
        let mut q = world.query::<&HsdShaderGraphMaterial>();
        let Some(hsd_mat) = q.iter(world).next() else {
            return false;
        };
        let assets = world.resource::<Assets<ShaderGraphMaterial>>();
        let Some(material) = assets.get(&hsd_mat.0) else {
            return false;
        };
        if material.params.inputs[0] == Vec4::new(1.0, 0.0, 0.0, 1.0) {
            handle = Some(hsd_mat.0.clone());
            return true;
        }
        false
    });

    handle.expect("overridden shader graph material");
}

/// Two prims referencing byte-identical compiled graphs share one generated
/// `Handle<Shader>`.
#[traced_test]
#[rstest]
fn test_shader_graph_shares_compiled_shader_across_prims(#[from(ctx_wds)] mut ctx: TestContext) {
    let bytes = glow_graph().encode().expect("encode graph");

    let a = ctx.create_prim();
    ctx.set_slot(a, slots::MATERIAL_GRAPH_DATA, bytes.clone());
    let b = ctx.create_prim();
    ctx.set_slot(b, slots::MATERIAL_GRAPH_DATA, bytes);

    let mut handles: Vec<Handle<ShaderGraphMaterial>> = Vec::new();
    ctx.tick_until(|world| {
        let mut q = world.query::<&HsdShaderGraphMaterial>();
        handles = q.iter(world).map(|m| m.0.clone()).collect();
        handles.len() == 2
    });

    let assets = ctx.app.world().resource::<Assets<ShaderGraphMaterial>>();
    let shaders: Vec<_> = handles
        .iter()
        .map(|h| assets.get(h).expect("material").fragment_shader.clone())
        .collect();
    assert_eq!(
        shaders[0], shaders[1],
        "identical graphs must share one compiled shader"
    );
}

/// Removing the slot removes the built material, as `ImageAttr`/`MaterialAttr`
/// removal does.
#[traced_test]
#[rstest]
fn test_shader_graph_removed_when_slot_removed(#[from(ctx_wds)] mut ctx: TestContext) {
    let bytes = glow_graph().encode().expect("encode graph");

    let prim = ctx.create_prim();
    ctx.set_slot(prim, slots::MATERIAL_GRAPH_DATA, bytes);

    ctx.tick_until(|world| {
        world
            .query::<&HsdShaderGraphMaterial>()
            .iter(world)
            .next()
            .is_some()
    });

    ctx.remove_slot(prim, slots::MATERIAL_GRAPH_DATA);
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut q = world.query::<&HsdMaterialGraphSlot>();
    assert!(q.iter(world).next().is_none());
}

/// A graph with a displacement network compiles and caches a vertex shader
/// too, not just a fragment one.
#[traced_test]
#[rstest]
fn test_shader_graph_with_displacement_compiles_a_vertex_shader(
    #[from(ctx_wds)] mut ctx: TestContext,
) {
    let graph = ShaderGraph {
        public_inputs: Vec::new(),
        surface:       SurfaceGraph {
            nodes: Vec::new(),
            output: SurfaceOutput::Unlit(UnlitOutput {
                color:                Port::Const(GraphValue::Color([1.0, 1.0, 1.0, 1.0])),
                alpha_clip_threshold: None,
            }),
            ..Default::default()
        },
        displacement:  Some(DisplacementGraph {
            nodes:                 vec![Node::LocalNormal, Node::Time],
            position_offset:       Some(Port::Node(0)),
            normal_override:       None,
            world_position_offset: None,
        }),
    };
    let bytes = graph.encode().expect("encode graph");

    let prim = ctx.create_prim();
    ctx.set_slot(prim, slots::MATERIAL_GRAPH_DATA, bytes);

    let mut handle: Option<Handle<ShaderGraphMaterial>> = None;
    ctx.tick_until(|world| {
        let mut q = world.query::<&HsdShaderGraphMaterial>();
        let Some(hsd_mat) = q.iter(world).next() else {
            return false;
        };
        handle = Some(hsd_mat.0.clone());
        true
    });

    let handle = handle.expect("material handle");
    let assets = ctx.app.world().resource::<Assets<ShaderGraphMaterial>>();
    let material = assets.get(&handle).expect("material asset");
    assert!(
        material.vertex_shader.is_some(),
        "a displacement network must compile a vertex shader"
    );
}

/// Blend and cull are declared by the graph, not inferred from which
/// terminals happen to be connected.
#[traced_test]
#[rstest]
fn blend_and_cull_reach_the_material(#[from(ctx_wds)] mut ctx: TestContext) {
    let graph = ShaderGraph {
        surface: SurfaceGraph {
            blend: BlendMode::Add,
            cull: CullMode::Front,
            ..Default::default()
        },
        ..Default::default()
    };
    let bytes = graph.encode().expect("encode graph");

    let prim = ctx.create_prim();
    ctx.set_slot(prim, slots::MATERIAL_GRAPH_DATA, bytes);

    let mut handle: Option<Handle<ShaderGraphMaterial>> = None;
    ctx.tick_until(|world| {
        let mut q = world.query::<&HsdShaderGraphMaterial>();
        let Some(hsd_mat) = q.iter(world).next() else {
            return false;
        };
        handle = Some(hsd_mat.0.clone());
        true
    });

    let handle = handle.expect("material handle");
    let assets = ctx.app.world().resource::<Assets<ShaderGraphMaterial>>();
    let material = assets.get(&handle).expect("material asset");
    assert!(matches!(material.alpha_mode, AlphaMode::Add));
    assert_eq!(material.cull_mode, Some(Face::Front));
}

/// One binding, two backends: `material:binding` names a prim, not a backend,
/// so a prim bound to one carrying a graph renders that graph and must not
/// also carry a competing `StandardMaterial`.
#[traced_test]
#[rstest]
fn binding_to_a_graph_prim_renders_that_graph(#[from(ctx_wds)] mut ctx: TestContext) {
    let bytes = glow_graph().encode().expect("encode graph");

    let template = ctx.create_prim();
    ctx.set_slot(template, slots::MATERIAL_GRAPH_DATA, bytes);

    let beam = ctx.create_prim();
    ctx.set_relationship(beam, material::BINDING, template);
    ctx.set_attr(
        beam,
        &GraphOverridesAttr {
            overrides: BTreeMap::from([(0, GraphValue::Color([1.0, 0.0, 0.0, 1.0]))]),
        },
    );

    let mut bound: Option<Handle<ShaderGraphMaterial>> = None;
    ctx.tick_until(|world| {
        let mut q = world.query::<(Entity, &HsdShaderGraphMaterial)>();
        let found = q.iter(world).count();
        if found < 2 {
            return false;
        }
        let mut q = world.query::<(&HsdShaderGraphMaterial, &ShaderGraphOverridesData)>();
        let Some((mat, _)) = q.iter(world).next() else {
            return false;
        };
        bound = Some(mat.0.clone());
        true
    });

    let bound = bound.expect("bound prim built a shader graph material");
    let world = ctx.app.world_mut();
    let assets = world.resource::<Assets<ShaderGraphMaterial>>();
    let material = assets.get(&bound).expect("material asset");
    assert_eq!(
        material.params.inputs[0],
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        "a graph binding shares the program but keeps this prim's own overrides"
    );

    let mut q = world.query::<&MeshMaterial3d<StandardMaterial>>();
    assert_eq!(
        q.iter(world).count(),
        0,
        "a prim rendered by a graph must not also carry a PBR material"
    );
}
