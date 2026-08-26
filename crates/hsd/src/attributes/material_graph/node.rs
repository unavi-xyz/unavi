use serde::{
    Deserialize,
    Serialize,
};

use super::value::{
    GraphValue,
    ValueKind,
};

/// A node input.
///
/// Either a constant baked into the compiled graph, a public input
/// overridable per-instance without recompiling, or another node's output.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Port {
    Const(GraphValue),
    /// Index into [`super::ShaderGraph::public_inputs`].
    Input(u16),
    /// Index into the enclosing network's node list. Must be strictly less
    /// than the index of the node this port belongs to, and never reaches
    /// across networks — surface and displacement run as different shader
    /// stages and share no per-invocation values.
    Node(u16),
}

/// A graph node.
///
/// Variants are appended, never reordered or removed: postcard encodes a
/// variant by its index, and a compiled graph's bytes are its content hash
/// and therefore its cache key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Node {
    Uv,
    WorldNormal,
    WorldPosition,
    VertexColor,
    LocalPosition,
    LocalNormal,
    Time,
    /// Either two operands of one kind, or a vector and a `Float`, which
    /// broadcasts across the vector's components.
    Add {
        a: Port,
        b: Port,
    },
    Mul {
        a: Port,
        b: Port,
    },
    Lerp {
        a: Port,
        b: Port,
        t: Port,
    },
    Dot {
        a: Port,
        b: Port,
    },
    Sin {
        x: Port,
    },
    Cos {
        x: Port,
    },
    /// Power exponent on `1 - dot(N, V)`; `N`/`V` are host-provided fragment
    /// vectors, not graph inputs. Surface-only: they are not defined in the
    /// vertex stage.
    Fresnel {
        power: Port,
    },
    Noise {
        uv: Port,
    },
    /// `slot` selects one of a fixed `MAX_TEXTURE_SAMPLES` texture bindings,
    /// never an open-ended name. Surface-only; see
    /// [`super::validate::error::GraphError::TextureSampleInDisplacement`].
    TextureSample {
        uv:   Port,
        slot: u8,
    },
    /// The only branching this format allows: a hard switch on `cond`, no
    /// general control flow.
    Select {
        cond: Port,
        a:    Port,
        b:    Port,
    },
    Sub {
        a: Port,
        b: Port,
    },
    Div {
        a: Port,
        b: Port,
    },
    OneMinus {
        x: Port,
    },
    Abs {
        x: Port,
    },
    Floor {
        x: Port,
    },
    Fract {
        x: Port,
    },
    Saturate {
        x: Port,
    },
    /// Negative inputs are clamped away rather than left to produce `NaN`.
    Sqrt {
        x: Port,
    },
    Pow {
        x: Port,
        y: Port,
    },
    Min {
        a: Port,
        b: Port,
    },
    Max {
        a: Port,
        b: Port,
    },
    Clamp {
        x:    Port,
        low:  Port,
        high: Port,
    },
    Step {
        edge: Port,
        x:    Port,
    },
    Smoothstep {
        low:  Port,
        high: Port,
        x:    Port,
    },
    Length {
        v: Port,
    },
    Normalize {
        v: Port,
    },
    Cross {
        a: Port,
        b: Port,
    },
    /// Reads one component out of a vector. The counterpart to the `Combine`
    /// nodes, and the reason this format needs no multi-output node: a
    /// [`Port::Node`] names a node, not one of several output ports.
    Extract {
        v:       Port,
        channel: u8,
    },
    Combine2 {
        x: Port,
        y: Port,
    },
    Combine3 {
        x: Port,
        y: Port,
        z: Port,
    },
    Combine4 {
        x: Port,
        y: Port,
        z: Port,
        w: Port,
    },
    /// Widens or narrows between vector kinds. Widening pads with zero,
    /// except that a widened [`ValueKind::Color`]'s alpha is 1.0.
    Convert {
        v:  Port,
        to: ValueKind,
    },
    /// A pseudo-random scalar in `0..1`, one value per draw instance: how
    /// instances sharing a graph can differ without a per-instance write.
    InstanceRandom,
    /// The prim's world-space origin.
    ObjectPosition,
    /// The prim's world-space scale, one component per local axis.
    ObjectScale,
    /// Unit vector from the surface toward the camera. Surface-only.
    ViewDirection,
    Atan2 {
        y: Port,
        x: Port,
    },
    /// WGSL's `%`, whose sign follows `a` — a remainder rather than a
    /// Euclidean modulo.
    Modulo {
        a: Port,
        b: Port,
    },
    Distance {
        a: Port,
        b: Port,
    },
    /// Rescales `x` from one range onto another, unbounded on both sides:
    /// clamping is [`Node::Saturate`]'s job.
    Remap {
        x:         Port,
        from_low:  Port,
        from_high: Port,
        to_low:    Port,
        to_high:   Port,
    },
    /// Rises from 0 to 1 and falls back over each unit of `x`.
    TriangleWave {
        x: Port,
    },
    /// Perceptual brightness, by the Rec. 709 weights. Alpha is ignored.
    Luminance {
        color: Port,
    },
    /// `uv` about `center`, as `(radius, angle)` with the angle normalised to
    /// `0..1` counterclockwise from +x.
    PolarCoords {
        uv:     Port,
        center: Port,
    },
    RotateUv {
        uv:      Port,
        center:  Port,
        radians: Port,
    },
    /// Where this fragment sits on screen, `0..1` from the top left.
    /// Surface-only: there is no screen before rasterization.
    ScreenUv,
    /// What was already drawn behind this surface, sampled at `uv`.
    ///
    /// Surface-only, and only what was drawn *before* this surface: two
    /// graphs reading it do not see each other, and neither sees anything
    /// that comes after.
    SceneColor {
        uv: Port,
    },
}

/// Which shader stage a network compiles to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Network {
    Surface,
    Displacement,
}
