use serde::{
    Deserialize,
    Serialize,
};

use super::{
    node::Port,
    value::GraphValue,
};

/// The fragment-stage network: two closed output shapes chosen at authoring
/// time, not a fully generic multi-output graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SurfaceGraph {
    pub nodes:        Vec<super::node::Node>,
    pub output:       SurfaceOutput,
    pub blend:        BlendMode,
    pub cull:         CullMode,
    /// Whether this surface occludes light. Declared rather than inferred
    /// from [`BlendMode`].
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
/// Declared, never inferred from which terminals happen to be connected.
/// Alpha *testing* is not a mode here — it is [`LitOutput::
/// alpha_clip_threshold`]/[`UnlitOutput::alpha_clip_threshold`], which
/// codegen emits as a `discard` and which composes with any blend mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    #[default]
    Opaque,
    Blend,
    Add,
    Multiply,
}

/// Which faces are discarded before rasterization. `Front` is what an
/// inverted-hull outline draws its extruded shell with.
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

/// Fed into a `PbrInput` before `apply_pbr_lighting`; codegen always targets
/// this one known shape when `Lit` is selected.
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
    /// Fraction of light transmitted and tinted by
    /// [`LitOutput::base_color`], `0..1`. Above zero routes the material into
    /// the transmissive phase, where Bevy refracts the already-drawn scene
    /// behind the surface by `thickness`/`ior`.
    pub specular_transmission: Option<Port>,
    /// The Lambertian transmitted lobe, lit from behind.
    pub diffuse_transmission:  Option<Port>,
    /// Metres the refracted ray travels inside the surface before exiting.
    pub thickness:             Option<Port>,
    /// Refractive index, `1.0` (air) being no refraction. Water is `1.33`,
    /// glass `1.5`.
    pub ior:                   Option<Port>,
}

/// Written straight to the fragment output; no `PbrInput`, no lighting pass.
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
/// `position_offset` is added to the mesh's local-space vertex position
/// before the standard local→world→clip transform; `normal_override`, if set,
/// replaces the local-space normal before it is transformed to world space.
///
/// `world_position_offset` is added after that transform instead, and
/// composes with `position_offset` rather than replacing it. It is the only
/// way to displace along a direction that does not rotate and scale with the
/// prim: the displacement network has no world-space leaf to build one from.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplacementGraph {
    pub nodes:                 Vec<super::node::Node>,
    pub position_offset:       Option<Port>,
    pub normal_override:       Option<Port>,
    pub world_position_offset: Option<Port>,
}
