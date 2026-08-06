use std::collections::BTreeMap;

use bevy::{
    pbr::MeshMaterial3d,
    prelude::*,
};
use bevy_hsd::attributes::material_graph::{
    HsdShaderGraphMaterial,
    ShaderGraphMaterial,
};
use hsd::attributes::{
    material_graph::{
        DisplacementGraph,
        GraphOverridesAttr,
        GraphValue,
        Node,
        NodeKind,
        Port,
        ShaderGraph,
        SurfaceGraph,
        SurfaceOutput,
        UnlitOutput,
    },
    slots,
};
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

/// Unlit rim glow: no lighting pass, so it needs no `PbrInput` at all — the
/// shape a beam/hologram/sky effect needs, unreachable through a fixed PBR
/// terminal set.
fn glow_graph() -> ShaderGraph {
    ShaderGraph {
        public_inputs: vec![GraphValue::Color([0.1, 0.6, 1.0, 1.0])],
        surface:       SurfaceGraph {
            nodes:  vec![
                Node {
                    kind: NodeKind::Fresnel {
                        power: Port::Const(GraphValue::Float(2.0)),
                    },
                },
                Node {
                    kind: NodeKind::Lerp {
                        a: Port::Const(GraphValue::Color([0.0, 0.0, 0.0, 1.0])),
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

/// No `GraphOverridesAttr` set: `HsdSlots` alone must be enough to trigger the
/// pipeline, since that attribute is optional and this is the common case —
/// see `HsdMaterialGraphSlot`'s docs.
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
    assert_eq!(material.alpha_mode, AlphaMode::Blend);
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

/// Two prims referencing byte-identical compiled graphs must end up sharing
/// one generated `Handle<Shader>` — the free dedup this format depends on,
/// now verified at the runtime cache rather than just at compile time.
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

/// Removing the slot removes the built material, mirroring how
/// `ImageAttr`/`MaterialAttr` removal already works.
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
    let mut q = world.query::<&bevy_hsd::attributes::material_graph::HsdMaterialGraphSlot>();
    assert!(q.iter(world).next().is_none());
}

/// A graph with a displacement network compiles and caches a vertex shader
/// too, not just a fragment one — the part a PBR-terminal-only design could
/// never express (physgun's curved beam needs exactly this).
#[traced_test]
#[rstest]
fn test_shader_graph_with_displacement_compiles_a_vertex_shader(
    #[from(ctx_wds)] mut ctx: TestContext,
) {
    let graph = ShaderGraph {
        public_inputs: Vec::new(),
        surface:       SurfaceGraph {
            nodes:  Vec::new(),
            output: SurfaceOutput::Unlit(UnlitOutput {
                color:                Port::Const(GraphValue::Color([1.0, 1.0, 1.0, 1.0])),
                alpha_clip_threshold: None,
            }),
        },
        displacement:  Some(DisplacementGraph {
            nodes:           vec![
                Node {
                    kind: NodeKind::LocalNormal,
                },
                Node {
                    kind: NodeKind::Time,
                },
            ],
            position_offset: Some(Port::Node(0)),
            normal_override: None,
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
