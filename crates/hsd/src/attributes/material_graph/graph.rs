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
    pub nodes:  Vec<super::node::Node>,
    pub output: SurfaceOutput,
}

impl Default for SurfaceGraph {
    fn default() -> Self {
        Self {
            nodes:  Vec::new(),
            output: SurfaceOutput::Unlit(UnlitOutput::default()),
        }
    }
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
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplacementGraph {
    pub nodes:           Vec<super::node::Node>,
    pub position_offset: Option<Port>,
    pub normal_override: Option<Port>,
}
