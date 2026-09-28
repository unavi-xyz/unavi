use serde::{
    Deserialize,
    Serialize,
};

use super::{
    node::Port,
    value::GraphValue,
};

/// The fragment-stage network.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SurfaceGraph {
    pub nodes:        Vec<super::node::Node>,
    pub output:       SurfaceOutput,
    pub blend:        BlendMode,
    pub cull:         CullMode,
    /// Not inferred from [`BlendMode`].
    pub cast_shadows: bool,
}

impl Default for SurfaceGraph {
    fn default() -> Self {
        Self {
            nodes:        Vec::new(),
            output:       SurfaceOutput::Unlit(UnlitOutput::default()),
            blend:        BlendMode::default(),
            cull:         CullMode::default(),
            cast_shadows: true,
        }
    }
}

/// How the fragment output composites against what is already there.
///
/// Alpha testing is a separate `alpha_clip_threshold` field, not a variant
/// here, and composes with any blend mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    #[default]
    Opaque,
    Blend,
    Add,
    Multiply,
}

/// Which faces are discarded before rasterization.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CullMode {
    #[default]
    Back,
    Front,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SurfaceOutput {
    Lit(LitOutput),
    Unlit(UnlitOutput),
}

/// Fed into a `PbrInput` before `apply_pbr_lighting`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LitOutput {
    pub base_color:            Option<Port>,
    pub emissive:              Option<Port>,
    pub metallic:              Option<Port>,
    pub roughness:             Option<Port>,
    pub normal:                Option<Port>,
    pub alpha:                 Option<Port>,
    /// When set, codegen emits a `discard` below this threshold.
    pub alpha_clip_threshold:  Option<Port>,
    /// Fraction of light transmitted, tinted by [`LitOutput::base_color`], in
    /// `0..1`.
    pub specular_transmission: Option<Port>,
    /// The Lambertian transmitted lobe, lit from behind.
    pub diffuse_transmission:  Option<Port>,
    /// Metres the refracted ray travels inside the surface before exiting.
    pub thickness:             Option<Port>,
    /// Refractive index. `1.0` (air) means no refraction.
    pub ior:                   Option<Port>,
}

/// Written straight to the fragment output. No `PbrInput`, no lighting pass.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UnlitOutput {
    pub color:                Port,
    pub alpha_clip_threshold: Option<Port>,
}

impl Default for UnlitOutput {
    fn default() -> Self {
        Self {
            color:                Port::Const(GraphValue::Color([1.0, 1.0, 1.0, 1.0])),
            alpha_clip_threshold: None,
        }
    }
}

/// The vertex-stage network.
///
/// `position_offset` applies before the local-to-world-to-clip transform.
/// `normal_override` replaces the local-space normal before that transform.
/// `world_position_offset` applies after the transform and composes with
/// `position_offset` rather than replacing it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplacementGraph {
    pub nodes:                 Vec<super::node::Node>,
    pub position_offset:       Option<Port>,
    pub normal_override:       Option<Port>,
    pub world_position_offset: Option<Port>,
}
