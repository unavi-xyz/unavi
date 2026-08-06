//! Renders `material:graph_data` (a compiled [`ShaderGraph`]) as a live
//! material, generating WGSL client-side from the validated graph.

pub mod codegen;

use bevy::{
    asset::uuid_handle,
    ecs::system::SystemParam,
    pbr::MeshMaterial3d,
    platform::collections::HashMap,
    prelude::*,
    render::render_resource::{
        AsBindGroup,
        ShaderType,
        SpecializedMeshPipelineError,
    },
};
use hsd::{
    attributes::{
        Attribute,
        material_graph::{
            GraphValue,
            ShaderGraph,
            SurfaceOutput,
            overrides::{
                GraphOverridesAttr,
                validate_overrides,
            },
            validate::validate,
        },
        slots,
    },
    id::BlobId,
};

use crate::{
    HsdChild,
    HsdRelationships,
    HsdSlots,
    attributes::{
        AttributeParser,
        ParseError,
        image::HsdImage,
    },
};

/// A shader without `#import`s to fall back on before any graph has loaded —
/// unlit black, so an unresolved material is visibly wrong rather than
/// silently reusing whatever the pipeline's own default happens to be.
/// [`ShaderGraphMaterial::specialize`] always overrides it once a graph
/// loads.
const FALLBACK_SHADER_HANDLE: Handle<Shader> = uuid_handle!("2f9e6f0a-7b1e-4d3a-9c0a-2f6b1a8e4d70");
const FALLBACK_SHADER_SOURCE: &str = "\
#import bevy_pbr::forward_io::VertexOutput

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
";

#[derive(Component, Clone)]
pub struct ShaderGraphOverridesData(pub GraphOverridesAttr);

pub struct ShaderGraphOverridesParser;

impl AttributeParser for ShaderGraphOverridesParser {
    fn key(&self) -> &'static str {
        GraphOverridesAttr::KEY
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        payload: Option<&[u8]>,
    ) -> Result<(), ParseError> {
        match payload {
            Some(payload) => {
                commands
                    .entity(prim)
                    .insert(ShaderGraphOverridesData(GraphOverridesAttr::decode(
                        payload,
                    )?));
            }
            None => {
                commands.entity(prim).remove::<ShaderGraphOverridesData>();
            }
        }
        Ok(())
    }
}

/// A prim's compiled-graph slot.
///
/// Unlike `image`/`mesh`, there is no always-present attribute a script must
/// set to mark "this prim has a shader graph" — [`GraphOverridesAttr`] is
/// optional, present only when a prim overrides a public input. Tracking
/// `HsdSlots` directly, the way `prefab` does, is the only trigger that does
/// not miss the common no-overrides case.
#[derive(Component, Debug, Clone)]
pub struct HsdMaterialGraphSlot(pub Vec<u8>);

pub fn track_material_graph(
    changed: Query<(Entity, &HsdSlots), Changed<HsdSlots>>,
    mut commands: Commands,
) {
    for (entity, slots) in &changed {
        match slots.0.get(slots::MATERIAL_GRAPH_DATA) {
            Some(bytes) => {
                commands
                    .entity(entity)
                    .insert(HsdMaterialGraphSlot(bytes.clone()));
            }
            None => {
                commands.entity(entity).remove::<HsdMaterialGraphSlot>();
            }
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct HsdShaderGraphMaterial(pub Handle<ShaderGraphMaterial>);

pub fn rebuild_material_graph(
    changed: Query<
        Entity,
        Or<(
            Changed<HsdMaterialGraphSlot>,
            Changed<ShaderGraphOverridesData>,
            Changed<HsdRelationships>,
        )>,
    >,
    slots: Query<&HsdMaterialGraphSlot>,
    overrides: Query<&ShaderGraphOverridesData>,
    texture_ctx: TextureCtx,
    time: Res<Time>,
    mut cache: ResMut<ShaderGraphCache>,
    mut shaders: ResMut<Assets<Shader>>,
    mut materials: ResMut<Assets<ShaderGraphMaterial>>,
    mut existing: Query<&mut HsdShaderGraphMaterial>,
    mut commands: Commands,
) {
    for prim in &changed {
        let Ok(slot) = slots.get(prim) else {
            continue;
        };

        let graph = match ShaderGraph::decode(&slot.0) {
            Ok(graph) => graph,
            Err(err) => {
                warn!(?err, "undecodable shader graph");
                continue;
            }
        };
        let validated = match validate(&graph) {
            Ok(validated) => validated,
            Err(err) => {
                warn!(?err, "invalid shader graph");
                continue;
            }
        };

        let overrides_attr = overrides.get(prim).ok().map(|o| &o.0);
        if let Some(overrides) = overrides_attr
            && let Err(err) = validate_overrides(&graph, overrides)
        {
            warn!(
                ?err,
                "shader graph overrides do not match the graph; using its defaults"
            );
        }
        let overrides_attr = overrides_attr.filter(|o| validate_overrides(&graph, o).is_ok());

        let hash = BlobId(*blake3::hash(&slot.0).as_bytes());
        let cached = cache.0.entry(hash).or_insert_with(|| {
            let fragment_source = codegen::generate_fragment_shader(&graph, &validated.surface);
            let fragment = shaders.add(Shader::from_wgsl(
                fragment_source,
                format!("generated://material_graph/{hash}/fragment"),
            ));
            let vertex = graph
                .displacement
                .as_ref()
                .zip(validated.displacement.as_ref())
                .map(|(displacement, kinds)| {
                    let source =
                        codegen::generate_vertex_shader(displacement, &graph.public_inputs, kinds);
                    shaders.add(Shader::from_wgsl(
                        source,
                        format!("generated://material_graph/{hash}/vertex"),
                    ))
                });
            CachedShaders { fragment, vertex }
        });

        let params = build_params(&graph, overrides_attr, time.elapsed_secs());
        let textures = resolve_textures(prim, &texture_ctx);

        let material = ShaderGraphMaterial {
            params,
            texture_0: textures[0].clone(),
            texture_1: textures[1].clone(),
            texture_2: textures[2].clone(),
            texture_3: textures[3].clone(),
            fragment_shader: cached.fragment.clone(),
            vertex_shader: cached.vertex.clone(),
            alpha_mode: alpha_mode(&graph.surface.output),
        };

        if let Ok(mut existing) = existing.get_mut(prim) {
            if let Some(mut asset) = materials.get_mut(&existing.0) {
                *asset = material;
            } else {
                existing.0 = materials.add(material);
            }
        } else {
            let handle = materials.add(material);
            commands.entity(prim).insert((
                HsdShaderGraphMaterial(handle.clone()),
                MeshMaterial3d(handle),
            ));
        }
    }
}

#[derive(SystemParam)]
pub struct TextureCtx<'w, 's> {
    children:      Query<'w, 's, &'static HsdChild>,
    indices:       Query<'w, 's, &'static crate::HsdPrimIndex>,
    relationships: Query<'w, 's, &'static HsdRelationships>,
    images:        Query<'w, 's, &'static HsdImage>,
}

/// Resolves up to [`codegen::MAX_TEXTURE_SAMPLES`] fixed texture slots by
/// relationship, mirroring `MaterialTextureRefs` — no blob fetch of its own,
/// since a referenced image prim's own `ImageParser`/`rebuild_image`
/// pipeline already loads it; this just reads the resulting handle.
fn resolve_textures(
    prim: Entity,
    ctx: &TextureCtx,
) -> [Option<Handle<Image>>; codegen::MAX_TEXTURE_SAMPLES] {
    let mut out: [Option<Handle<Image>>; codegen::MAX_TEXTURE_SAMPLES] = Default::default();

    let Ok(doc_child) = ctx.children.get(prim) else {
        return out;
    };
    let Ok(index) = ctx.indices.get(doc_child.0) else {
        return out;
    };
    let Ok(rels) = ctx.relationships.get(prim) else {
        return out;
    };

    for (slot, handle) in out.iter_mut().enumerate() {
        let name = slots::material_graph_texture(slot as u8);
        *handle = rels
            .0
            .get(name.as_str())
            .and_then(|target| index.0.get(target))
            .and_then(|&ent| ctx.images.get(ent).ok())
            .map(|img| img.0.clone());
    }

    out
}

const fn pack_input(value: GraphValue) -> Vec4 {
    match value {
        GraphValue::Float(v) => Vec4::new(v, 0.0, 0.0, 0.0),
        GraphValue::Vec2([x, y]) => Vec4::new(x, y, 0.0, 0.0),
        GraphValue::Vec3([x, y, z]) => Vec4::new(x, y, z, 0.0),
        GraphValue::Color([r, g, b, a]) => Vec4::new(r, g, b, a),
    }
}

fn build_params(
    graph: &ShaderGraph,
    overrides: Option<&GraphOverridesAttr>,
    time: f32,
) -> GraphParams {
    let mut inputs = [Vec4::ZERO; codegen::MAX_PUBLIC_INPUTS];
    for (index, default) in graph
        .public_inputs
        .iter()
        .enumerate()
        .take(codegen::MAX_PUBLIC_INPUTS)
    {
        let value = overrides
            .and_then(|o| o.overrides.get(&(index as u16)))
            .filter(|v| v.kind() == default.kind())
            .copied()
            .unwrap_or(*default);
        inputs[index] = pack_input(value);
    }
    GraphParams { inputs, time }
}

/// A graph's compiled shaders. Every prim referencing the same graph reuses
/// these — the fragment/vertex `Handle<Shader>`s are cached keyed by the
/// slot's own content hash, free since that hash already uniquely
/// identifies the compiled graph bytes.
#[derive(Clone)]
struct CachedShaders {
    fragment: Handle<Shader>,
    /// `None` when the graph has no
    /// [`hsd::attributes::material_graph::DisplacementGraph`] — the mesh
    /// pipeline's own default vertex shader is used instead.
    vertex:   Option<Handle<Shader>>,
}

#[derive(Resource, Default)]
pub struct ShaderGraphCache(HashMap<BlobId, CachedShaders>);

/// Unity's Alpha Clip Threshold / Unreal's Opacity Mask maps to
/// `AlphaMode::Mask`; an explicit `alpha`/`Unlit` output that can be
/// translucent maps to `Blend`; otherwise `Opaque`. A heuristic inferred from
/// which terminals are set, not an explicit graph field the format asks an
/// author to choose.
const fn alpha_mode(output: &SurfaceOutput) -> AlphaMode {
    match output {
        SurfaceOutput::Lit(lit) => {
            if lit.alpha_clip_threshold.is_some() {
                AlphaMode::Mask(0.5)
            } else if lit.alpha.is_some() {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            }
        }
        SurfaceOutput::Unlit(unlit) => {
            if unlit.alpha_clip_threshold.is_some() {
                AlphaMode::Mask(0.5)
            } else {
                AlphaMode::Blend
            }
        }
    }
}

/// Advances the graph's `time` public leaf every frame.
///
/// Mirrors `SeamMaterial`'s per-frame uniform update — a plain `get_mut` on
/// every tick would flag every shader-graph material dirty and force a GPU
/// re-upload each frame regardless of whether anything else changed.
pub fn advance_material_graph_time(
    time: Res<Time>,
    graphs: Query<&HsdShaderGraphMaterial>,
    mut materials: ResMut<Assets<ShaderGraphMaterial>>,
) {
    let now = time.elapsed_secs();
    for graph in &graphs {
        if let Some(mut material) = materials.get_mut(&graph.0) {
            material.params.time = now;
        }
    }
}

#[derive(Clone, Copy, ShaderType, Debug, Default, PartialEq)]
pub struct GraphParams {
    pub inputs: [Vec4; codegen::MAX_PUBLIC_INPUTS],
    pub time:   f32,
}

/// Fixed-budget `AsBindGroup`.
///
/// One static Rust type with a generous uniform buffer and a fixed
/// texture-slot array, so a graph cannot express more live state than the
/// format's own caps already allow.
#[derive(Asset, AsBindGroup, Clone, TypePath)]
#[bind_group_data(ShaderGraphMaterialKey)]
pub struct ShaderGraphMaterial {
    #[uniform(0)]
    pub params:          GraphParams,
    #[texture(1)]
    #[sampler(2)]
    pub texture_0:       Option<Handle<Image>>,
    #[texture(3)]
    #[sampler(4)]
    pub texture_1:       Option<Handle<Image>>,
    #[texture(5)]
    #[sampler(6)]
    pub texture_2:       Option<Handle<Image>>,
    #[texture(7)]
    #[sampler(8)]
    pub texture_3:       Option<Handle<Image>>,
    pub fragment_shader: Handle<Shader>,
    /// `None` for a graph with no displacement network — the mesh
    /// pipeline's own default vertex shader runs unmodified.
    pub vertex_shader:   Option<Handle<Shader>>,
    pub alpha_mode:      AlphaMode,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ShaderGraphMaterialKey {
    fragment_shader: Handle<Shader>,
    vertex_shader:   Option<Handle<Shader>>,
}

impl From<&ShaderGraphMaterial> for ShaderGraphMaterialKey {
    fn from(material: &ShaderGraphMaterial) -> Self {
        Self {
            fragment_shader: material.fragment_shader.clone(),
            vertex_shader:   material.vertex_shader.clone(),
        }
    }
}

impl Material for ShaderGraphMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        FALLBACK_SHADER_HANDLE.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    /// Every graph shares one `Material` type; what makes each look
    /// different is which generated `Handle<Shader>`s get bound here, keyed
    /// off [`ShaderGraphMaterialKey`] so distinct graphs specialize into
    /// distinct pipelines.
    ///
    /// The vertex swap is nested inside the fragment guard, not independent
    /// of it: a depth-only prepass/shadow pass has no fragment stage, and
    /// also specializes its `Vertex` input without the `normal`/`uv` fields
    /// our generated vertex shader unconditionally reads (those shader defs
    /// are only set when something in the pass actually needs them, which a
    /// depth-only pass doesn't). Swapping the vertex shader there produces
    /// an invalid-field-accessor shader-compile error, not a silent
    /// fallback — so displacement is main-pass-only for v1; shadows are
    /// cast from the undisplaced mesh.
    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader = key.bind_group_data.fragment_shader;
            if let Some(vertex_shader) = key.bind_group_data.vertex_shader {
                descriptor.vertex.shader = vertex_shader;
            }
        }
        Ok(())
    }
}

pub fn register_fallback_shader(mut shaders: ResMut<Assets<Shader>>) {
    shaders
        .insert(
            &FALLBACK_SHADER_HANDLE,
            Shader::from_wgsl(
                FALLBACK_SHADER_SOURCE,
                "generated://material_graph/fallback",
            ),
        )
        .expect("fallback shader handle is a fixed uuid, inserted once at startup");
}
