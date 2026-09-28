use std::{
    collections::BTreeMap,
    io::Cursor,
};

use bevy::{
    pbr::MeshMaterial3d,
    prelude::*,
    render::render_resource::Face,
};
use bevy_hsd::attributes::{
    image::HsdImage,
    shader::{
        HsdMaterialGraphSlot,
        HsdShaderGraphMaterial,
        MAX_SHADER_PROGRAMS,
        ShaderGraphMaterial,
        ShaderGraphOverridesData,
    },
};
use hsd::{
    attributes::{
        material,
        shader::{
            self,
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
    },
    property::Payload,
};
use image::{
    ImageFormat,
    RgbaImage,
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

/// A graph whose bytes, and so whose content hash and cache key, differ per
/// `step`.
fn distinct_graph(step: usize) -> ShaderGraph {
    let mut graph = glow_graph();
    graph.public_inputs = vec![GraphValue::Float(step as f32)];
    graph.surface.nodes = vec![Node::Mul {
        a: Port::Input(0),
        b: Port::Const(GraphValue::Color([1.0, 1.0, 1.0, 1.0])),
    }];
    graph.surface.output = SurfaceOutput::Unlit(UnlitOutput {
        color:                Port::Node(0),
        alpha_clip_threshold: None,
    });
    graph
}

/// No `GraphOverridesAttr` set: the graph attribute alone must trigger the
/// pipeline, since overrides are optional and this is the common case.
#[traced_test]
#[rstest]
fn test_shader_graph_without_overrides(#[from(ctx_blobs)] mut ctx: TestContext) {
    let bytes = glow_graph().encode().expect("encode graph");

    let prim = ctx.create_prim();
    ctx.set_shader_graph(prim, bytes);

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
fn test_shader_graph_with_overrides(#[from(ctx_blobs)] mut ctx: TestContext) {
    let bytes = glow_graph().encode().expect("encode graph");

    let prim = ctx.create_prim();
    ctx.set_shader_graph(prim, bytes);
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
fn test_shader_graph_shares_compiled_shader_across_prims(#[from(ctx_blobs)] mut ctx: TestContext) {
    let bytes = glow_graph().encode().expect("encode graph");

    let a = ctx.create_prim();
    ctx.set_shader_graph(a, bytes.clone());
    let b = ctx.create_prim();
    ctx.set_shader_graph(b, bytes);

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

/// A compiled program is a pure function of the graph's bytes, so it is not
/// the document's to own: keying the cache by document would compile identical
/// WGSL once per document and specialize a second pipeline for it.
#[traced_test]
#[rstest]
fn a_graph_compiles_once_across_documents(#[from(ctx_blobs)] mut ctx: TestContext) {
    let bytes = glow_graph().encode().expect("encode graph");

    let here = ctx.create_prim();
    ctx.set_shader_graph(here, bytes.clone());

    let elsewhere = ctx.spawn_document();
    let there = elsewhere.create_prim();
    elsewhere.set_shader_graph(there, bytes);

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
    assert_eq!(shaders[0], shaders[1]);
}

/// The cap is a per-document resource ceiling, so one document exhausting it
/// must not spend another's budget.
#[traced_test]
#[rstest]
fn the_program_cap_is_charged_per_document(#[from(ctx_blobs)] mut ctx: TestContext) {
    for step in 0..MAX_SHADER_PROGRAMS {
        let prim = ctx.create_prim();
        ctx.set_shader_graph(prim, distinct_graph(step).encode().expect("encode graph"));
    }
    ctx.tick_until(|world| {
        world.query::<&HsdShaderGraphMaterial>().iter(world).count() == MAX_SHADER_PROGRAMS
    });

    let elsewhere = ctx.spawn_document();
    let there = elsewhere.create_prim();
    elsewhere.set_shader_graph(
        there,
        distinct_graph(MAX_SHADER_PROGRAMS)
            .encode()
            .expect("encode graph"),
    );

    ctx.tick_until(|world| {
        world.query::<&HsdShaderGraphMaterial>().iter(world).count() == MAX_SHADER_PROGRAMS + 1
    });
}

/// Now that a program is shared, dropping one document must not evict a graph
/// another still holds — the next prim to ask for it would otherwise compile a
/// second copy and specialize a second pipeline for identical WGSL.
///
/// Asserted by asking for it again rather than by reading the cache: a
/// material holds its own strong `Handle<Shader>`, so the asset stays alive
/// whether or not the cache still has it and its liveness proves nothing.
#[traced_test]
#[rstest]
fn dropping_one_document_keeps_a_graph_another_still_holds(
    #[from(ctx_blobs)] mut ctx: TestContext,
) {
    let bytes = glow_graph().encode().expect("encode graph");

    let kept = ctx.create_prim();
    ctx.set_shader_graph(kept, bytes.clone());

    let leaving = ctx.spawn_document();
    let doomed = leaving.create_prim();
    leaving.set_shader_graph(doomed, bytes.clone());

    ctx.tick_until(|world| world.query::<&HsdShaderGraphMaterial>().iter(world).count() == 2);
    let before = fragment_shaders(&mut ctx).pop().expect("a compiled shader");

    ctx.despawn_document(&leaving);
    ctx.app.update();

    let again = ctx.create_prim();
    ctx.set_shader_graph(again, bytes);
    ctx.tick_until(|world| world.query::<&HsdShaderGraphMaterial>().iter(world).count() == 2);

    let after = fragment_shaders(&mut ctx);
    assert_eq!(after.len(), 2);
    assert!(
        after.iter().all(|shader| *shader == before),
        "the graph was evicted with the other document and recompiled"
    );
}

fn fragment_shaders(ctx: &mut TestContext) -> Vec<Handle<Shader>> {
    let world = ctx.app.world_mut();
    let handles: Vec<_> = world
        .query::<&HsdShaderGraphMaterial>()
        .iter(world)
        .map(|m| m.0.clone())
        .collect();
    let materials = world.resource::<Assets<ShaderGraphMaterial>>();
    handles
        .iter()
        .map(|h| materials.get(h).expect("material").fragment_shader.clone())
        .collect()
}

/// Removing the graph attribute removes the built material, as
/// `image/data`/`material/value` removal does.
#[traced_test]
#[rstest]
fn test_shader_graph_removed_when_removed(#[from(ctx_blobs)] mut ctx: TestContext) {
    let bytes = glow_graph().encode().expect("encode graph");

    let prim = ctx.create_prim();
    ctx.set_shader_graph(prim, bytes);

    ctx.tick_until(|world| {
        world
            .query::<&HsdShaderGraphMaterial>()
            .iter(world)
            .next()
            .is_some()
    });

    ctx.remove_attr::<ShaderGraph>(prim);
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
    #[from(ctx_blobs)] mut ctx: TestContext,
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
    ctx.set_shader_graph(prim, bytes);

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
fn blend_and_cull_reach_the_material(#[from(ctx_blobs)] mut ctx: TestContext) {
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
    ctx.set_shader_graph(prim, bytes);

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

/// One binding, two backends: `material/binding` names a prim, not a backend,
/// so a prim bound to one carrying a graph renders that graph and must not
/// also carry a competing `StandardMaterial`.
#[traced_test]
#[rstest]
fn binding_to_a_graph_prim_renders_that_graph(#[from(ctx_blobs)] mut ctx: TestContext) {
    let bytes = glow_graph().encode().expect("encode graph");

    let template = ctx.create_prim();
    ctx.set_shader_graph(template, bytes);

    let beam = ctx.create_prim();
    ctx.set_relationship(beam, &material::BINDING, template);
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

/// Editing the source prim's graph after another prim binds to it rebuilds
/// the bound prim too, not just the source.
#[traced_test]
#[rstest]
fn editing_the_source_graph_after_binding_rebuilds_the_bound_prim(
    #[from(ctx_blobs)] mut ctx: TestContext,
) {
    let template = ctx.create_prim();
    ctx.set_shader_graph(template, glow_graph().encode().expect("encode graph"));

    let beam = ctx.create_prim();
    ctx.set_relationship(beam, &material::BINDING, template);
    ctx.app.update();
    let beam_entity = ctx.prim_entity(ctx.doc, beam);

    let mut before: Option<Handle<Shader>> = None;
    ctx.tick_until(|world| {
        let Some(mat) = world.get::<HsdShaderGraphMaterial>(beam_entity) else {
            return false;
        };
        let assets = world.resource::<Assets<ShaderGraphMaterial>>();
        let Some(material) = assets.get(&mat.0) else {
            return false;
        };
        before = Some(material.fragment_shader.clone());
        true
    });
    let before = before.expect("beam built a shader graph material");

    ctx.set_shader_graph(template, distinct_graph(0).encode().expect("encode graph"));

    ctx.tick_until(|world| {
        let mat = world
            .get::<HsdShaderGraphMaterial>(beam_entity)
            .expect("beam still has a material");
        let assets = world.resource::<Assets<ShaderGraphMaterial>>();
        let material = assets.get(&mat.0).expect("material asset");
        material.fragment_shader != before
    });
}

/// Setting an image prim's data after a graph sampling it has already built
/// still fills the texture slot, rather than leaving it empty forever.
#[traced_test]
#[rstest]
fn setting_image_data_after_the_graph_is_built_fills_the_texture_slot(
    #[from(ctx_blobs)] mut ctx: TestContext,
) {
    let image_prim = ctx.create_prim();

    let graph = ShaderGraph {
        surface: SurfaceGraph {
            nodes: vec![Node::TextureSample {
                uv:   Port::Const(GraphValue::Vec2([0.0, 0.0])),
                slot: 0,
            }],
            output: SurfaceOutput::Unlit(UnlitOutput {
                color:                Port::Node(0),
                alpha_clip_threshold: None,
            }),
            ..Default::default()
        },
        ..Default::default()
    };

    let material_prim = ctx.create_prim();
    ctx.set_shader_graph(material_prim, graph.encode().expect("encode graph"));
    ctx.set_relationship(
        material_prim,
        &shader::texture(0).expect("texture slot 0"),
        image_prim,
    );

    ctx.tick_until(|world| {
        world
            .query::<&HsdShaderGraphMaterial>()
            .iter(world)
            .next()
            .is_some()
    });

    let mut rgba = RgbaImage::new(2, 2);
    rgba.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
    let mut png_bytes = Vec::new();
    image::DynamicImage::ImageRgba8(rgba)
        .write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
        .expect("encode png");
    ctx.set_image_data(image_prim, png_bytes);

    let mut handle: Option<Handle<Image>> = None;
    ctx.tick_until(|world| {
        let Some(image) = world
            .query::<&HsdImage>()
            .iter(world)
            .next()
            .map(|i| i.0.clone())
        else {
            return false;
        };
        let mut q = world.query::<&HsdShaderGraphMaterial>();
        let Some(mat) = q.iter(world).next() else {
            return false;
        };
        let assets = world.resource::<Assets<ShaderGraphMaterial>>();
        let Some(material) = assets.get(&mat.0) else {
            return false;
        };
        if material.texture_0.as_ref() == Some(&image) {
            handle = Some(image);
            return true;
        }
        false
    });
    handle.expect("texture slot filled after the image decoded");
}
