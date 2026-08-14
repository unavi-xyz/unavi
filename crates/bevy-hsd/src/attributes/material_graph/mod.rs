//! Renders `material:graph_data` (a compiled [`ShaderGraph`]) as a live
//! material, generating WGSL client-side from the validated graph.

pub mod codegen;

use bevy::{
    asset::uuid_handle,
    ecs::system::SystemParam,
    light::NotShadowCaster,
    pbr::MeshMaterial3d,
    platform::collections::{
        HashMap,
        HashSet,
    },
    prelude::*,
    render::render_resource::{
        AsBindGroup,
        Face,
        ShaderType,
        SpecializedMeshPipelineError,
    },
};
use hsd::{
    attributes::{
        Attribute,
        material_graph::{
            MAX_PUBLIC_INPUTS,
            MAX_TEXTURE_SAMPLES,
            ShaderGraph,
            graph::{
                BlendMode,
                CullMode,
            },
            overrides::{
                GraphOverridesAttr,
                validate_overrides,
            },
            validate::validate,
            value::GraphValue,
        },
        slots,
    },
    id::BlobId,
};

use crate::{
    Hsd,
    HsdChild,
    HsdRelationships,
    HsdSlots,
    attributes::{
        AttributeParser,
        ParseError,
        image::HsdImage,
        material_source::MaterialSource,
    },
};

/// Fallback before any graph has loaded: unlit black, so an unresolved
/// material is visibly wrong rather than silently reusing the pipeline's
/// default. [`ShaderGraphMaterial::specialize`] overrides it once a graph
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
/// `GraphOverridesAttr` is optional, present only when a prim overrides a
/// public input, so tracking `HsdSlots` directly is the only trigger that
/// catches the common no-overrides case.
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

/// The graph's own public-input defaults, kept so an override change can be
/// applied without decoding and re-validating the graph again.
///
/// Overrides change per-frame while the graph behind them rarely does, so the
/// two must not share a rebuild path.
#[derive(Component, Debug, Clone)]
pub struct GraphInputDefaults(pub Vec<GraphValue>);

/// Writes changed overrides straight into the existing material's uniform
/// block, skipping decode, validation and codegen entirely.
///
/// A prim whose graph is still being built has no [`GraphInputDefaults`] yet
/// and is picked up by [`rebuild_material_graph`] instead.
pub fn apply_graph_overrides(
    changed: Query<
        (
            &HsdShaderGraphMaterial,
            &GraphInputDefaults,
            Option<&ShaderGraphOverridesData>,
        ),
        Changed<ShaderGraphOverridesData>,
    >,
    mut materials: ResMut<Assets<ShaderGraphMaterial>>,
) {
    for (material, defaults, overrides) in &changed {
        let params = build_params_from(&defaults.0, overrides.map(|o| &o.0));
        let Some(mut asset) = materials.get_mut(&material.0) else {
            continue;
        };
        // Guarded: `get_mut` marks the asset changed regardless, and a changed
        // material re-uploads its whole bind group.
        if asset.params != params {
            asset.params = params;
        }
    }
}

pub fn rebuild_material_graph(
    changed: Query<
        Entity,
        Or<(
            Changed<HsdMaterialGraphSlot>,
            Changed<HsdRelationships>,
            Changed<MaterialSource>,
        )>,
    >,
    sources: Query<&MaterialSource>,
    slots: Query<&HsdMaterialGraphSlot>,
    overrides: Query<&ShaderGraphOverridesData>,
    doc_of: Query<&HsdChild>,
    texture_ctx: TextureCtx,
    mut cache: ResMut<ShaderGraphCache>,
    mut shaders: ResMut<Assets<Shader>>,
    mut materials: ResMut<Assets<ShaderGraphMaterial>>,
    mut existing: Query<&mut HsdShaderGraphMaterial>,
    mut commands: Commands,
) {
    for prim in &changed {
        // The graph may live on another prim: `material:binding` names a prim,
        // and a bound prim renders the target's graph with its own overrides.
        let Ok(&MaterialSource::Graph(source)) = sources.get(prim) else {
            continue;
        };
        let Ok(slot) = slots.get(source) else {
            continue;
        };
        let Ok(doc) = doc_of.get(prim).map(|c| c.0) else {
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

        let overrides_attr = overrides.get(prim).ok().map(|o| &o.0).filter(|o| {
            match validate_overrides(&graph, o) {
                Ok(()) => true,
                Err(err) => {
                    warn!(
                        ?err,
                        "shader graph overrides do not match the graph; using its defaults"
                    );
                    false
                }
            }
        });

        let hash = BlobId(*blake3::hash(&slot.0).as_bytes());
        let compile = || {
            let fragment_source = codegen::generate_fragment_shader(&graph, &validated);
            let fragment = shaders.add(Shader::from_wgsl(
                fragment_source,
                format!("generated://material_graph/{hash}/fragment"),
            ));
            let vertex = codegen::generate_vertex_shader(&graph, &validated).map(|source| {
                shaders.add(Shader::from_wgsl(
                    source,
                    format!("generated://material_graph/{hash}/vertex"),
                ))
            });
            CachedShaders { fragment, vertex }
        };
        let Some(cached) = cache.charge(doc, hash, compile) else {
            warn!(
                "document is at its cap of {MAX_SHADER_PROGRAMS} shader programs; ignoring another"
            );
            continue;
        };

        let params = build_params(&graph, overrides_attr);
        let textures = resolve_textures(prim, &texture_ctx);

        let material = ShaderGraphMaterial {
            params,
            texture_0: textures[0].clone(),
            texture_1: textures[1].clone(),
            texture_2: textures[2].clone(),
            texture_3: textures[3].clone(),
            fragment_shader: cached.fragment.clone(),
            vertex_shader: cached.vertex.clone(),
            alpha_mode: alpha_mode(graph.surface.blend),
            cull_mode: cull_mode(graph.surface.cull),
        };

        commands
            .entity(prim)
            .insert(GraphInputDefaults(graph.public_inputs.clone()));

        if graph.surface.cast_shadows {
            commands.entity(prim).remove::<NotShadowCaster>();
        } else {
            commands.entity(prim).insert(NotShadowCaster);
        }

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

/// Resolves up to [`MAX_TEXTURE_SAMPLES`] fixed texture slots by relationship.
/// The referenced image prim's own pipeline already loads the image; this
/// reads the resulting handle.
fn resolve_textures(
    prim: Entity,
    ctx: &TextureCtx,
) -> [Option<Handle<Image>>; MAX_TEXTURE_SAMPLES] {
    let mut out: [Option<Handle<Image>>; MAX_TEXTURE_SAMPLES] = Default::default();

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

fn build_params(graph: &ShaderGraph, overrides: Option<&GraphOverridesAttr>) -> GraphParams {
    build_params_from(&graph.public_inputs, overrides)
}

fn build_params_from(
    defaults: &[GraphValue],
    overrides: Option<&GraphOverridesAttr>,
) -> GraphParams {
    let mut inputs = [Vec4::ZERO; MAX_PUBLIC_INPUTS];
    for (index, default) in defaults.iter().enumerate().take(MAX_PUBLIC_INPUTS) {
        let value = overrides
            .and_then(|o| o.overrides.get(&(index as u16)))
            .filter(|v| v.kind() == default.kind())
            .copied()
            .unwrap_or(*default);
        inputs[index] = pack_input(value);
    }
    GraphParams { inputs }
}

/// A graph's compiled shaders, keyed by the slot's content hash, so every prim
/// referencing the same graph reuses them.
#[derive(Clone)]
struct CachedShaders {
    fragment: Handle<Shader>,
    /// `None` when the graph has no
    /// [`hsd::attributes::material_graph::DisplacementGraph`] — the mesh
    /// pipeline's own default vertex shader is used instead.
    vertex:   Option<Handle<Shader>>,
}

/// Distinct compiled graphs one document may hold at once.
///
/// Each is a shader asset and a specialized render pipeline that lives until
/// the document does; a graph's hash changes with any edit, so without a
/// ceiling a document that varies one constant mints them without bound.
pub const MAX_SHADER_PROGRAMS: usize = 32;

/// Compiled shaders, and which documents asked for them.
///
/// The two are separate because they answer different questions. A program is
/// a pure function of the graph's bytes, so two documents carrying the same
/// graph should compile it **once** — keying the programs by document would
/// compile it per document and specialize a second pipeline for an identical
/// shader. The cap, meanwhile, is a per-document resource ceiling and has to
/// stay one: a global count would let one document exhaust the budget for
/// every other.
#[derive(Resource, Default)]
pub struct ShaderGraphCache {
    programs: HashMap<BlobId, CachedShaders>,
    /// The graphs each document has charged against its cap. Also what keeps
    /// a program alive: one is dropped when the last document holding it
    /// goes.
    ///
    /// A document never gives a graph back, even once no prim renders it —
    /// the cap exists to bound edit churn, and re-charging on every hash
    /// change would leave it bounding nothing.
    charged:  HashMap<Entity, HashSet<BlobId>>,
}

impl ShaderGraphCache {
    /// The shaders for `hash`, compiled by `compile` if this is the first
    /// document to ask for them, and charged against `doc`'s cap.
    ///
    /// `None` once `doc` is at [`MAX_SHADER_PROGRAMS`] distinct graphs and
    /// `hash` is not already one of them.
    fn charge(
        &mut self,
        doc: Entity,
        hash: BlobId,
        compile: impl FnOnce() -> CachedShaders,
    ) -> Option<&CachedShaders> {
        let charged = self.charged.entry(doc).or_default();
        if !charged.contains(&hash) {
            if charged.len() >= MAX_SHADER_PROGRAMS {
                return None;
            }
            charged.insert(hash);
        }
        Some(self.programs.entry(hash).or_insert_with(compile))
    }
}

pub fn evict_document_shaders(trigger: On<Remove, Hsd>, mut cache: ResMut<ShaderGraphCache>) {
    let Some(dropped) = cache.charged.remove(&trigger.entity) else {
        return;
    };
    let ShaderGraphCache { programs, charged } = &mut *cache;
    programs.retain(|hash, _| {
        !dropped.contains(hash) || charged.values().any(|held| held.contains(hash))
    });
}

const fn alpha_mode(blend: BlendMode) -> AlphaMode {
    match blend {
        BlendMode::Opaque => AlphaMode::Opaque,
        BlendMode::Blend => AlphaMode::Blend,
        BlendMode::Add => AlphaMode::Add,
        BlendMode::Multiply => AlphaMode::Multiply,
    }
}

/// `None` is wgpu's "cull nothing"; [`CullMode::Back`] is the default a graph
/// gets, and only an explicit [`CullMode::None`] disables culling.
const fn cull_mode(cull: CullMode) -> Option<Face> {
    match cull {
        CullMode::Back => Some(Face::Back),
        CullMode::Front => Some(Face::Front),
        CullMode::None => None,
    }
}

#[derive(Clone, Copy, ShaderType, Debug, Default, PartialEq)]
pub struct GraphParams {
    pub inputs: [Vec4; MAX_PUBLIC_INPUTS],
}

/// Fixed-budget `AsBindGroup`: one static Rust type with a generous uniform
/// buffer and a fixed texture-slot array, so a graph cannot express more live
/// state than the format's own caps allow.
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
    pub cull_mode:       Option<Face>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ShaderGraphMaterialKey {
    fragment_shader: Handle<Shader>,
    vertex_shader:   Option<Handle<Shader>>,
    cull_mode:       Option<Face>,
}

impl From<&ShaderGraphMaterial> for ShaderGraphMaterialKey {
    fn from(material: &ShaderGraphMaterial) -> Self {
        Self {
            fragment_shader: material.fragment_shader.clone(),
            vertex_shader:   material.vertex_shader.clone(),
            cull_mode:       material.cull_mode,
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

    /// Every graph shares one `Material` type; what makes each look different
    /// is which generated `Handle<Shader>`s get bound here, keyed off
    /// [`ShaderGraphMaterialKey`] so distinct graphs specialize into distinct
    /// pipelines.
    ///
    /// The vertex swap stays inside the fragment guard: a depth-only prepass
    /// or shadow pass has no fragment stage and specializes its `Vertex`
    /// input without the `normal`/`uv` fields the generated vertex shader
    /// unconditionally reads. Swapping the vertex shader there yields an
    /// invalid-field-accessor compile error, not a silent fallback, so
    /// displacement is main-pass-only and shadows cast from the undisplaced
    /// mesh.
    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = key.bind_group_data.cull_mode;
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
