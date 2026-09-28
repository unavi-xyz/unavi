//! Renders `shader/graph` (a compiled [`ShaderGraph`]) as a live material,
//! generating WGSL client-side from the validated graph.

pub mod codegen;

use bevy::{
    light::NotShadowCaster,
    mesh::MeshVertexBufferLayoutRef,
    pbr::{
        MaterialPipeline,
        MaterialPipelineKey,
        MeshMaterial3d,
    },
    platform::collections::{
        HashMap,
        HashSet,
    },
    prelude::*,
    render::render_resource::{
        AsBindGroup,
        Face,
        RenderPipelineDescriptor,
        ShaderType,
        SpecializedMeshPipelineError,
    },
};
use hsd::{
    attributes::shader::{
        self,
        MAX_PUBLIC_INPUTS,
        MAX_TEXTURE_SAMPLES,
        ShaderGraph,
        graph::{
            BlendMode,
            CullMode,
            SurfaceOutput,
        },
        node,
        overrides::{
            GraphOverridesAttr,
            validate_overrides,
        },
        validate::validate,
        value::GraphValue,
    },
    property::{
        Payload,
        name::PropName,
    },
};

use crate::{
    Hsd,
    HsdChild,
    HsdRelationships,
    attributes::{
        ParseError,
        image::HsdImage,
        material_source::MaterialSource,
        relations::PrimRelations,
    },
};

/// Unlit black, so an unresolved material is visibly wrong rather than
/// silently reusing the pipeline's default. [`ShaderGraphMaterial::specialize`]
/// overrides it once a graph loads.
const FALLBACK_SHADER: &str = "embedded://bevy_hsd/attributes/shader/fallback.wgsl";

#[derive(Component, Clone)]
pub struct ShaderGraphOverridesData(pub GraphOverridesAttr);

/// A prim's compiled graph. The hash is computed once at parse time so a
/// cache hit never has to decode the bytes.
#[derive(Component, Debug, Clone)]
pub struct HsdMaterialGraphSlot {
    hash:  blake3::Hash,
    bytes: Vec<u8>,
}

/// One group holds both the graph and its per-instance overrides, so one
/// handler dispatches on the field it was called for.
pub(crate) fn apply(
    commands: &mut Commands,
    prim: Entity,
    name: &PropName,
    payload: Option<&[u8]>,
) -> Result<(), ParseError> {
    match name.field() {
        Some("graph") => match payload {
            Some(payload) => {
                commands.entity(prim).insert(HsdMaterialGraphSlot {
                    hash:  blake3::hash(payload),
                    bytes: payload.to_vec(),
                });
            }
            None => {
                commands.entity(prim).remove::<HsdMaterialGraphSlot>();
            }
        },
        Some("overrides") => {
            match payload {
                Some(payload) => {
                    commands.entity(prim).insert(ShaderGraphOverridesData(
                        GraphOverridesAttr::decode(payload)?,
                    ));
                }
                None => {
                    commands.entity(prim).remove::<ShaderGraphOverridesData>();
                }
            }
        }
        _ => {}
    }
    Ok(())
}

#[derive(Component, Debug, Clone)]
pub struct HsdShaderGraphMaterial(pub Handle<ShaderGraphMaterial>);

/// The cache hash a prim's material was last built from, so an overrides-only
/// edit can look up the graph's public inputs without recompiling.
#[derive(Component, Clone, Copy)]
pub(crate) struct BuiltFromGraph(blake3::Hash);

/// Writes changed overrides straight into the existing material's uniform
/// block, skipping decode, validation and codegen entirely.
pub(crate) fn apply_graph_overrides(
    changed: Query<
        (
            &HsdShaderGraphMaterial,
            &BuiltFromGraph,
            Option<&ShaderGraphOverridesData>,
        ),
        Changed<ShaderGraphOverridesData>,
    >,
    cache: Res<ShaderGraphCache>,
    mut materials: ResMut<Assets<ShaderGraphMaterial>>,
) {
    for (material, built, overrides) in &changed {
        let Some(cached) = cache.get(built.0) else {
            continue;
        };
        let params = build_params_from(&cached.public_inputs, overrides.map(|o| &o.0));
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

pub(crate) fn rebuild_shader_material(
    changed: Query<
        Entity,
        Or<(
            Changed<HsdMaterialGraphSlot>,
            Changed<HsdRelationships>,
            Changed<MaterialSource>,
        )>,
    >,
    changed_slots: Query<Entity, Changed<HsdMaterialGraphSlot>>,
    changed_images: Query<Entity, Changed<HsdImage>>,
    sources: Query<(Entity, &MaterialSource)>,
    slots: Query<&HsdMaterialGraphSlot>,
    overrides: Query<&ShaderGraphOverridesData>,
    doc_of: Query<&HsdChild>,
    relations: PrimRelations,
    images: Query<&HsdImage>,
    mut cache: ResMut<ShaderGraphCache>,
    mut shaders: ResMut<Assets<Shader>>,
    mut materials: ResMut<Assets<ShaderGraphMaterial>>,
    mut existing: Query<&mut HsdShaderGraphMaterial>,
    mut commands: Commands,
) {
    let mut dirty: HashSet<Entity> = changed.iter().collect();
    mark_binders_of_changed_graphs(&mut dirty, &changed_slots, &sources);
    mark_graphs_with_changed_textures(&mut dirty, &changed_images, &sources, &relations);

    for prim in dirty {
        // The graph may live on another prim: `material/binding` names a prim,
        // and a bound prim renders the target's graph with its own overrides.
        let Ok((_, &MaterialSource::Graph(source))) = sources.get(prim) else {
            continue;
        };
        let Ok(slot) = slots.get(source) else {
            continue;
        };
        let Ok(&HsdChild(doc)) = doc_of.get(prim) else {
            continue;
        };

        let Some(cached) = cache.get_or_build(slot.hash, &slot.bytes, doc, &mut shaders) else {
            continue;
        };

        let overrides_attr = overrides.get(prim).ok().map(|o| &o.0).filter(|o| {
            match validate_overrides(&cached.public_inputs, o) {
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

        let params = build_params_from(&cached.public_inputs, overrides_attr);
        let [texture_0, texture_1, texture_2, texture_3] =
            resolve_textures(prim, &relations, &images);

        let material = ShaderGraphMaterial {
            params,
            texture_0,
            texture_1,
            texture_2,
            texture_3,
            fragment_shader: cached.fragment.clone(),
            vertex_shader: cached.vertex.clone(),
            alpha_mode: cached.alpha_mode,
            cull_mode: cached.cull_mode,
            reads_scene: cached.reads_scene,
            transmissive: cached.transmissive,
        };

        commands.entity(prim).insert(BuiltFromGraph(slot.hash));
        if cached.cast_shadows {
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

/// A prim bound to a source whose graph slot just changed renders that
/// source's program, and must rebuild with it.
fn mark_binders_of_changed_graphs(
    dirty: &mut HashSet<Entity>,
    changed_slots: &Query<Entity, Changed<HsdMaterialGraphSlot>>,
    sources: &Query<(Entity, &MaterialSource)>,
) {
    if changed_slots.is_empty() {
        return;
    }
    let changed: HashSet<Entity> = changed_slots.iter().collect();
    for (prim, source) in sources {
        if let MaterialSource::Graph(target) = source
            && changed.contains(target)
        {
            dirty.insert(prim);
        }
    }
}

/// A graph prim sampling a texture prim whose `HsdImage` just changed rebuilds
/// too, since the handle it samples moved.
fn mark_graphs_with_changed_textures(
    dirty: &mut HashSet<Entity>,
    changed_images: &Query<Entity, Changed<HsdImage>>,
    sources: &Query<(Entity, &MaterialSource)>,
    relations: &PrimRelations,
) {
    if changed_images.is_empty() {
        return;
    }
    let changed: HashSet<Entity> = changed_images.iter().collect();
    for (prim, source) in sources {
        if !matches!(source, MaterialSource::Graph(_)) {
            continue;
        }
        let samples_changed = (0..MAX_TEXTURE_SAMPLES).any(|slot| {
            shader::texture(slot as u8)
                .and_then(|name| relations.target(prim, &name))
                .is_some_and(|target| changed.contains(&target))
        });
        if samples_changed {
            dirty.insert(prim);
        }
    }
}

/// Resolves up to [`MAX_TEXTURE_SAMPLES`] fixed texture slots by relationship.
/// The referenced image prim's own pipeline already loads the image; this
/// reads the resulting handle.
fn resolve_textures(
    prim: Entity,
    relations: &PrimRelations,
    images: &Query<&HsdImage>,
) -> [Option<Handle<Image>>; MAX_TEXTURE_SAMPLES] {
    let mut out: [Option<Handle<Image>>; MAX_TEXTURE_SAMPLES] = Default::default();

    for (slot, handle) in out.iter_mut().enumerate() {
        let Some(name) = shader::texture(slot as u8) else {
            continue;
        };
        *handle = relations
            .target(prim, &name)
            .and_then(|ent| images.get(ent).ok())
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

/// A compiled graph's material inputs, cached by content hash so a hit needs
/// no decode: everything a build needs comes off this value.
#[derive(Clone)]
struct CachedGraph {
    fragment:      Handle<Shader>,
    /// `None` when the graph has no
    /// [`hsd::schema::shader::graph::DisplacementGraph`] — the mesh
    /// pipeline's own default vertex shader is used instead.
    vertex:        Option<Handle<Shader>>,
    public_inputs: Vec<GraphValue>,
    alpha_mode:    AlphaMode,
    cull_mode:     Option<Face>,
    cast_shadows:  bool,
    reads_scene:   bool,
    transmissive:  bool,
}

/// Distinct compiled graphs one document may hold at once.
///
/// Each is a shader asset and a specialized render pipeline that lives until
/// the document does; a graph's hash changes with any edit, so without a
/// ceiling a document that varies one constant mints them without bound.
pub const MAX_SHADER_PROGRAMS: usize = 32;

/// Compiled graphs, keyed by the graph bytes' hash, so identical graphs
/// across documents compile once.
#[derive(Resource, Default)]
pub struct ShaderGraphCache {
    programs: HashMap<blake3::Hash, CachedGraph>,
    /// The graphs each document has charged against its cap; also what keeps
    /// a program alive: one is dropped when the last document holding it
    /// goes.
    charged:  HashMap<Entity, HashSet<blake3::Hash>>,
}

impl ShaderGraphCache {
    fn get(&self, hash: blake3::Hash) -> Option<&CachedGraph> {
        self.programs.get(&hash)
    }

    /// The cached entry for `hash`, decoding, validating and compiling
    /// `bytes` only on a miss, and charging `doc`'s cap only for a build
    /// that succeeds.
    ///
    /// `None` when the graph is undecodable or invalid, or `doc` is at its
    /// cap of [`MAX_SHADER_PROGRAMS`] and `hash` is new to it.
    fn get_or_build(
        &mut self,
        hash: blake3::Hash,
        bytes: &[u8],
        doc: Entity,
        shaders: &mut Assets<Shader>,
    ) -> Option<CachedGraph> {
        if let Some(cached) = self.get(hash) {
            let cached = cached.clone();
            return self.charge(doc, hash).then_some(cached);
        }

        let graph = ShaderGraph::decode(bytes)
            .inspect_err(|err| warn!(?err, "undecodable shader graph"))
            .ok()?;
        let validated = validate(&graph)
            .inspect_err(|err| warn!(?err, "invalid shader graph"))
            .ok()?;
        let fragment_source = codegen::generate_fragment_shader(&graph, &validated)
            .inspect_err(|err| warn!(?err, "failed to generate fragment shader"))
            .ok()?;
        let vertex_source = codegen::generate_vertex_shader(&graph, &validated)
            .inspect_err(|err| warn!(?err, "failed to generate vertex shader"))
            .ok()?;

        let fragment = shaders.add(Shader::from_wgsl(
            fragment_source,
            format!("generated://shader/{hash}/fragment"),
        ));
        let vertex = vertex_source.map(|source| {
            shaders.add(Shader::from_wgsl(
                source,
                format!("generated://shader/{hash}/vertex"),
            ))
        });

        let built = CachedGraph {
            fragment,
            vertex,
            public_inputs: graph.public_inputs.clone(),
            alpha_mode: alpha_mode(graph.surface.blend),
            cull_mode: cull_mode(graph.surface.cull),
            cast_shadows: graph.surface.cast_shadows,
            reads_scene: reads_scene(&graph),
            transmissive: transmissive(&graph),
        };

        if !self.charge(doc, hash) {
            warn!(
                "document is at its cap of {MAX_SHADER_PROGRAMS} shader programs; ignoring another"
            );
            return None;
        }
        self.programs.entry(hash).or_insert(built.clone());
        Some(built)
    }

    /// Charges `hash` against `doc`'s cap if it is not already charged.
    /// `false` once `doc` is at [`MAX_SHADER_PROGRAMS`] distinct graphs and
    /// `hash` is not already one of them.
    fn charge(&mut self, doc: Entity, hash: blake3::Hash) -> bool {
        let charged = self.charged.entry(doc).or_default();
        if charged.contains(&hash) {
            return true;
        }
        if charged.len() >= MAX_SHADER_PROGRAMS {
            return false;
        }
        charged.insert(hash);
        true
    }
}

pub(crate) fn evict_document_shaders(
    trigger: On<Remove, Hsd>,
    mut cache: ResMut<ShaderGraphCache>,
) {
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

/// `None` is wgpu's "cull nothing".
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
    /// Whether the graph reads what was drawn behind it, which is what a
    /// refraction needs and the only reason to pay for the transmissive pass.
    pub reads_scene:     bool,
    /// Whether the surface asks Bevy's PBR for transmissive glass: refraction
    /// through `thickness`/`ior` with depth rejection. Also what opts the
    /// material into the depth prepass.
    pub transmissive:    bool,
}

/// Whether the graph samples `SceneColor` itself — a hand-rolled screen-space
/// refraction rather than Bevy's transmissive material.
fn reads_scene(graph: &ShaderGraph) -> bool {
    graph
        .surface
        .nodes
        .iter()
        .any(|n| matches!(n, node::Node::SceneColor { .. }))
}

/// Whether the lit surface asks for Bevy's PBR transmissive glass.
const fn transmissive(graph: &ShaderGraph) -> bool {
    let SurfaceOutput::Lit(lit) = &graph.surface.output else {
        return false;
    };
    lit.specular_transmission.is_some() || lit.diffuse_transmission.is_some()
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ShaderGraphMaterialKey {
    fragment_shader: Handle<Shader>,
    vertex_shader:   Option<Handle<Shader>>,
    cull_mode:       Option<Face>,
    transmissive:    bool,
}

impl From<&ShaderGraphMaterial> for ShaderGraphMaterialKey {
    fn from(material: &ShaderGraphMaterial) -> Self {
        Self {
            fragment_shader: material.fragment_shader.clone(),
            vertex_shader:   material.vertex_shader.clone(),
            cull_mode:       material.cull_mode,
            transmissive:    material.transmissive,
        }
    }
}

impl Material for ShaderGraphMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        FALLBACK_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    /// Moves the material into the transmissive phase, which is drawn after
    /// the opaque one into a texture the fragment stage can then sample. Only
    /// when the graph needs it: the phase costs a full-screen copy.
    fn reads_view_transmission_texture(&self) -> bool {
        self.reads_scene || self.transmissive
    }

    /// Excluded so the transmissive depth rejection never compares a thin
    /// shell's refracted sample against its own prepass depth, which would
    /// read as "in front" and render the glass opaque.
    fn enable_prepass() -> bool {
        false
    }

    /// Every graph shares one `Material` type; the generated `Handle<Shader>`s
    /// bound here are what makes each look different, keyed off
    /// [`ShaderGraphMaterialKey`] so distinct graphs specialize into distinct
    /// pipelines.
    ///
    /// The generated shaders are written against `forward_io`, whose vertex
    /// attribute and interpolant locations differ from the `prepass_io` layout
    /// a prepass or shadow pipeline is built with, so they belong to the main
    /// pass alone: displacement is main-pass-only and shadows cast from the
    /// undisplaced mesh.
    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = key.bind_group_data.cull_mode;
        if descriptor
            .vertex
            .shader_defs
            .contains(&"PREPASS_PIPELINE".into())
        {
            return Ok(());
        }
        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader = key.bind_group_data.fragment_shader;
            if let Some(vertex_shader) = key.bind_group_data.vertex_shader {
                descriptor.vertex.shader = vertex_shader;
            }
            // `apply_pbr_lighting` compiles the transmissive path only under
            // this def.
            if key.bind_group_data.transmissive {
                fragment
                    .shader_defs
                    .push("STANDARD_MATERIAL_SPECULAR_TRANSMISSION".into());
            }
        }
        Ok(())
    }
}
