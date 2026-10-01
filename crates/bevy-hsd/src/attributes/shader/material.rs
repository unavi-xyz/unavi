use bevy::{
    mesh::MeshVertexBufferLayoutRef,
    pbr::{
        MaterialPipeline,
        MaterialPipelineKey,
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
use hsd::attributes::shader::{
    MAX_PUBLIC_INPUTS,
    ShaderGraph,
    graph::{
        BlendMode,
        CullMode,
        SurfaceOutput,
    },
    node,
};

/// Unlit black, so an unresolved material is visibly wrong rather than
/// silently reusing the pipeline's default. [`ShaderGraphMaterial::specialize`]
/// overrides it once a graph loads.
const FALLBACK_SHADER: &str = "embedded://bevy_hsd/attributes/shader/fallback.wgsl";

#[derive(Clone, Copy, ShaderType, Debug, Default, PartialEq)]
pub struct GraphParams {
    pub inputs: [Vec4; MAX_PUBLIC_INPUTS],
}

/// One static Rust type backs every graph, with a generous uniform buffer
/// and a fixed texture-slot array. A graph cannot express more live state
/// than this format allows.
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
    /// `None` for a graph with no displacement network. The mesh pipeline's
    /// own default vertex shader runs unmodified.
    pub vertex_shader:   Option<Handle<Shader>>,
    pub alpha_mode:      AlphaMode,
    pub cull_mode:       Option<Face>,
    /// Whether the graph samples what was drawn behind it, as opposed to
    /// Bevy's transmissive material.
    pub reads_scene:     bool,
    /// Whether the surface asks Bevy's PBR for transmissive glass, refraction
    /// through `thickness`/`ior` with depth rejection. Also opts the material
    /// into the depth prepass.
    pub transmissive:    bool,
}

/// Whether the graph samples `SceneColor` itself — a hand-rolled screen-space
/// refraction rather than Bevy's transmissive material.
pub(crate) fn reads_scene(graph: &ShaderGraph) -> bool {
    graph
        .surface
        .nodes
        .iter()
        .any(|n| matches!(n, node::Node::SceneColor { .. }))
}

/// Whether the lit surface asks for Bevy's PBR transmissive glass.
pub(crate) const fn transmissive(graph: &ShaderGraph) -> bool {
    let SurfaceOutput::Lit(lit) = &graph.surface.output else {
        return false;
    };
    lit.specular_transmission.is_some() || lit.diffuse_transmission.is_some()
}

pub(crate) const fn alpha_mode(blend: BlendMode) -> AlphaMode {
    match blend {
        BlendMode::Opaque => AlphaMode::Opaque,
        BlendMode::Blend => AlphaMode::Blend,
        BlendMode::Add => AlphaMode::Add,
        BlendMode::Multiply => AlphaMode::Multiply,
    }
}

/// `None` is wgpu's "cull nothing".
pub(crate) const fn cull_mode(cull: CullMode) -> Option<Face> {
    match cull {
        CullMode::Back => Some(Face::Back),
        CullMode::Front => Some(Face::Front),
        CullMode::None => None,
    }
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

    /// This phase costs a full-screen copy, so it only runs when the graph
    /// needs it.
    fn reads_view_transmission_texture(&self) -> bool {
        self.reads_scene || self.transmissive
    }

    /// Returns false because the transmissive depth rejection would otherwise
    /// compare a thin shell's refracted sample to its own prepass depth,
    /// reading it as in front and rendering the glass opaque.
    fn enable_prepass() -> bool {
        false
    }

    /// Generated shaders use the `forward_io` layout, so prepass and shadow
    /// pipelines keep their defaults. Displacement is main-pass only.
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
