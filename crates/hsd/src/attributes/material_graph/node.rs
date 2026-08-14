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
/// The zero-arity leaves are the only way a graph reaches shader-stage
/// context; each is legal in exactly one network (enforced by
/// [`super::validate::validate`], not by the type — see
/// [`super::validate::error::GraphError::WrongNetwork`]): `Uv`/`WorldNormal`/
/// `WorldPosition`/`VertexColor`/`ViewDirection` are surface-only
/// (fragment-stage varyings), `LocalPosition`/`LocalNormal` are
/// displacement-only (vertex-stage attributes), and `Time`/`InstanceRandom`/
/// `ObjectPosition`/`ObjectScale` are legal in both.
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
    /// Pairs either two operands of one kind, or a vector and a `Float`,
    /// which broadcasts across the vector's components. WGSL's own `+ - * /`
    /// already accept mixed scalar/vector operands, so this costs codegen
    /// nothing; the builtin-backed nodes ([`Node::Min`], [`Node::Step`], …)
    /// have no such rule and require matching kinds.
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
    /// Legal in both networks — the basic oscillator a `Time`-driven pulse,
    /// wave or sway effect needs, in either the fragment or vertex stage.
    Sin {
        x: Port,
    },
    Cos {
        x: Port,
    },
    /// Power exponent on `1 - dot(N, V)`. `N`/`V` are the fragment's own
    /// normal/view vectors, not graph inputs — a graph cannot construct an
    /// arbitrary Fresnel term, only parameterize the host-provided one.
    /// Surface-only: `N`/`V` are not defined in the vertex stage.
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
    /// general control flow. Matches how GPUs actually execute (SIMT).
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
    /// Negative inputs are clamped away rather than left to produce `NaN`,
    /// following [`Node::Fresnel`]'s existing clamp.
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
    /// except that a widened [`ValueKind::Color`]'s alpha is 1.0 — a
    /// zero-padded color would be fully transparent, which no caller wants.
    Convert {
        v:  Port,
        to: ValueKind,
    },
    /// A pseudo-random scalar in `0..1`, one value per draw instance.
    ///
    /// The only way instances sharing a graph can differ without a
    /// per-instance write. Noise cannot stand in: every instance of one mesh
    /// samples it at the same coordinates and so gets the same value, and the
    /// coordinates that do differ per instance are positions, which move.
    /// Unreal's `PerInstanceRandom` and Godot's `INSTANCE_CUSTOM` are the
    /// same node.
    InstanceRandom,
    /// The prim's world-space origin.
    ObjectPosition,
    /// The prim's world-space scale, one component per local axis. What a
    /// term measured in world units divides by to stay a fixed size while
    /// the prim it is drawn on is scaled.
    ObjectScale,
    /// The unit vector from the surface toward the camera. Surface-only, and
    /// the general form of the view term [`Node::Fresnel`] hardcodes.
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
    /// Rescales `x` from one range onto another. Unbounded on both sides:
    /// clamping is [`Node::Saturate`]'s job, and a remap that clamped could
    /// not extrapolate.
    Remap {
        x:         Port,
        from_low:  Port,
        from_high: Port,
        to_low:    Port,
        to_high:   Port,
    },
    /// Rises from 0 to 1 and falls back over each unit of `x`. The
    /// non-sinusoidal oscillator, for a scan or sweep that should travel at
    /// an even rate rather than easing at its extremes.
    TriangleWave {
        x: Port,
    },
    /// Perceptual brightness, by the Rec. 709 weights. Alpha is ignored.
    Luminance {
        color: Port,
    },
    /// `uv` about `center`, as `(radius, angle)` with the angle normalised to
    /// `0..1` counterclockwise from +x. What a radial sweep or a swirl reads
    /// instead of building `atan2` by hand.
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
    /// The one node that reads the frame rather than the mesh, and what makes
    /// refraction expressible: offsetting `uv` by a surface's own curvature
    /// bends whatever is behind it. Surface-only, and only ever what was drawn
    /// *before* this surface — so two graphs reading it do not see each other,
    /// and neither sees anything that comes after.
    SceneColor {
        uv: Port,
    },
}

/// Which shader stage a network compiles to. Carried in errors so a
/// validation failure names the network it came from; also what
/// [`super::validate::validate`] checks each leaf [`Node`] against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Network {
    Surface,
    Displacement,
}
