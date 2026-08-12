use serde::{
    Deserialize,
    Serialize,
};

use super::{
    node::Port,
    value::GraphValue,
};

/// The fragment-stage network.
///
/// Two closed shapes, chosen at authoring time, not a fully generic
/// multi-output graph — mirrors Unity Shader Graph's Lit/Unlit Master Stack
/// targets and Unreal's Shading Model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SurfaceGraph {
    pub nodes:        Vec<super::node::Node>,
    pub output:       SurfaceOutput,
    pub blend:        BlendMode,
    pub cull:         CullMode,
    /// Whether this surface occludes light. Declared rather than inferred
    /// from [`BlendMode`]: blending does not decide shadowing.
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
/// Declared, never inferred from which terminals happen to be connected:
/// USD's `opacityMode` is explicit for the same reason, and an inferred
/// mode silently puts every unlit graph in the transparent queue.
/// Alpha *testing* is not a mode here — it is [`LitOutput::
/// alpha_clip_threshold`]/[`UnlitOutput::alpha_clip_threshold`], which
/// codegen emits as a `discard` and which composes with any blend mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    #[default]
    Opaque,
    Blend,
    /// What makes a beam, hologram or any energy effect read as light
    /// rather than as tinted glass.
    Add,
    Multiply,
}

/// Which faces are discarded before rasterization.
///
/// `Front` is what an inverted-hull outline is: draw an outward-extruded
/// copy of a mesh with its front faces gone, so only the shell peeking past
/// the original silhouette survives the depth test.
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

/// Fed into a `PbrInput` before `apply_pbr_lighting`, mirroring glTF/PBR's
/// small standard surface-output vocabulary — codegen always targets this
/// one known shape when `Lit` is selected.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LitOutput {
    pub base_color:           Option<Port>,
    pub emissive:             Option<Port>,
    pub metallic:             Option<Port>,
    pub roughness:            Option<Port>,
    pub normal:               Option<Port>,
    pub alpha:                Option<Port>,
    /// Unity's Alpha Clip Threshold / Unreal's Opacity Mask: when set,
    /// codegen emits a `discard` below this threshold — still a single
    /// bounded statement, not a loop or general branch.
    pub alpha_clip_threshold: Option<Port>,
}

/// Written straight to the fragment output; no `PbrInput`, no lighting pass.
/// What `SkyMaterial`, a beam, a hologram, or any additive/emissive VFX
/// needs, and what a `Lit`-only terminal set cannot express.
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
/// before the standard local→world→clip transform runs; `normal_override`,
/// if set, replaces the local-space normal before it is transformed to
/// world space. Covers displacement, billboarding, and simple vertex
/// animation; the Bevy-side vertex shader splices this in right after
/// fetching the mesh's local-space position/normal, before the standard
/// mesh transform runs (see `bevy_pbr`'s own vertex shader for that part).
///
/// `world_position_offset` (Unreal's World Position Offset) is added after
/// that transform instead, and composes with `position_offset` rather than
/// replacing it. It is the only way to displace along a direction that does
/// not rotate and scale with the prim: a prim stretched between two points
/// has no local-space vector that stays world-down, and the displacement
/// network has no world-space leaf to build one from.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplacementGraph {
    pub nodes:                 Vec<super::node::Node>,
    pub position_offset:       Option<Port>,
    pub normal_override:       Option<Port>,
    pub world_position_offset: Option<Port>,
}
