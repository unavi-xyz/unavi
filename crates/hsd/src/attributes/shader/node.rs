use serde::{
    Deserialize,
    Serialize,
};

use super::value::{
    GraphValue,
    ValueKind,
};

/// A node input.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Port {
    Const(GraphValue),
    /// Index into [`super::ShaderGraph::public_inputs`].
    Input(u16),
    /// Index into the enclosing network's node list. Must be strictly less
    /// than the index of the node this port belongs to, and never reaches
    /// across networks.
    Node(u16),
}

/// A graph node.
///
/// Variants are appended, never reordered or removed. Postcard encodes a
/// variant by its index, and a compiled graph's bytes are its content hash
/// and cache key.
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
    /// Power exponent on `1 - dot(N, V)`, where `N`/`V` are host-provided
    /// fragment vectors. Surface-only.
    Fresnel {
        power: Port,
    },
    Noise {
        uv: Port,
    },
    /// `slot` selects one of a fixed `MAX_TEXTURE_SAMPLES` texture bindings,
    /// not an open-ended name. Surface-only.
    TextureSample {
        uv:   Port,
        slot: u8,
    },
    /// The only branching this format allows. A hard switch on `cond`, no
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
    /// Negative inputs are clamped, never producing `NaN`.
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
    /// Reads one component out of a vector.
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
    /// A pseudo-random scalar in `0..1`, one value per draw instance.
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
    /// WGSL's `%`. Sign follows `a`, not a Euclidean modulo.
    Modulo {
        a: Port,
        b: Port,
    },
    Distance {
        a: Port,
        b: Port,
    },
    /// Rescales `x` from one range onto another, unbounded. Clamping is
    /// [`Node::Saturate`]'s job.
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
    /// Surface-only.
    ScreenUv,
    /// What was already drawn behind this surface, sampled at `uv`.
    ///
    /// Surface-only. Two surfaces reading this do not see each other's
    /// output, and neither sees anything drawn after.
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
