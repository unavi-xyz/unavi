//! `wired:shading/graph`, lowering a script-built shader graph the same way
//! `bindings::native::shading` does, onto the same `hsd` types. No node
//! carries a byte or numeric list, so every shape here round-trips through
//! `serde` directly.

use std::rc::Rc;

use hsd::attributes::shader::{
    ShaderGraph,
    graph::{
        BlendMode,
        CullMode,
        DisplacementGraph,
        LitOutput,
        SurfaceGraph,
        SurfaceOutput,
        UnlitOutput,
    },
    node::{
        Node,
        Port,
    },
    value::GraphValue,
};
use serde::{
    Deserialize,
    Serialize,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use super::{
    Runtime,
    convert::{
        self,
        WireColor,
        WireVec2,
        WireVec3,
        from_js,
        to_js,
    },
    scene::DocumentHandle,
};
use crate::{
    error::ScriptError,
    host::shading,
};

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(tag = "tag", content = "val", rename_all = "kebab-case")]
enum WireGraphValue {
    Float(f32),
    Vec2(WireVec2),
    Vec3(WireVec3),
    Color(WireColor),
}

const fn graph_value(v: WireGraphValue) -> GraphValue {
    match v {
        WireGraphValue::Float(v) => GraphValue::Float(v),
        WireGraphValue::Vec2(v) => GraphValue::Vec2([v.x, v.y]),
        WireGraphValue::Vec3(v) => GraphValue::Vec3([v.x, v.y, v.z]),
        WireGraphValue::Color(c) => GraphValue::Color([c.r, c.g, c.b, c.a]),
    }
}

const fn wire_graph_value(value: GraphValue) -> WireGraphValue {
    match value {
        GraphValue::Float(v) => WireGraphValue::Float(v),
        GraphValue::Vec2([x, y]) => WireGraphValue::Vec2(WireVec2 { x, y }),
        GraphValue::Vec3([x, y, z]) => WireGraphValue::Vec3(WireVec3 { x, y, z }),
        GraphValue::Color([r, g, b, a]) => WireGraphValue::Color(WireColor { r, g, b, a }),
    }
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(tag = "tag", content = "val", rename_all = "kebab-case")]
enum WirePort {
    Const(WireGraphValue),
    Input(u16),
    Node(u16),
}

const fn port(p: WirePort) -> Port {
    match p {
        WirePort::Const(v) => Port::Const(graph_value(v)),
        WirePort::Input(i) => Port::Input(i),
        WirePort::Node(i) => Port::Node(i),
    }
}

#[derive(Deserialize, Serialize, Clone, Copy)]
struct WireBinaryOp {
    a: WirePort,
    b: WirePort,
}
#[derive(Deserialize, Serialize, Clone, Copy)]
struct WireLerpOp {
    a: WirePort,
    b: WirePort,
    t: WirePort,
}
#[derive(Deserialize, Serialize, Clone, Copy)]
struct WirePowOp {
    x: WirePort,
    y: WirePort,
}
#[derive(Deserialize, Serialize, Clone, Copy)]
struct WireAtan2Op {
    y: WirePort,
    x: WirePort,
}
#[derive(Deserialize, Serialize, Clone, Copy)]
struct WireSelectOp {
    cond: WirePort,
    a:    WirePort,
    b:    WirePort,
}
#[derive(Deserialize, Serialize, Clone, Copy)]
struct WireClampOp {
    x:    WirePort,
    low:  WirePort,
    high: WirePort,
}
#[derive(Deserialize, Serialize, Clone, Copy)]
struct WireStepOp {
    edge: WirePort,
    x:    WirePort,
}
#[derive(Deserialize, Serialize, Clone, Copy)]
struct WireSmoothstepOp {
    low:  WirePort,
    high: WirePort,
    x:    WirePort,
}
#[derive(Deserialize, Serialize, Clone, Copy)]
struct WireTextureSampleOp {
    uv:   WirePort,
    slot: u8,
}
#[derive(Deserialize, Serialize, Clone, Copy)]
struct WireExtractOp {
    v:       WirePort,
    channel: u8,
}
#[derive(Deserialize, Serialize, Clone, Copy)]
struct WireCombine2Op {
    x: WirePort,
    y: WirePort,
}
#[derive(Deserialize, Serialize, Clone, Copy)]
struct WireCombine3Op {
    x: WirePort,
    y: WirePort,
    z: WirePort,
}
#[derive(Deserialize, Serialize, Clone, Copy)]
struct WireCombine4Op {
    x: WirePort,
    y: WirePort,
    z: WirePort,
    w: WirePort,
}
#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct WireConvertOp {
    v:  WirePort,
    to: WireValueKind,
}
#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct WireRemapOp {
    x:         WirePort,
    from_low:  WirePort,
    from_high: WirePort,
    to_low:    WirePort,
    to_high:   WirePort,
}
#[derive(Deserialize, Serialize, Clone, Copy)]
struct WirePolarCoordsOp {
    uv:     WirePort,
    center: WirePort,
}
#[derive(Deserialize, Serialize, Clone, Copy)]
struct WireRotateUvOp {
    uv:      WirePort,
    center:  WirePort,
    radians: WirePort,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum WireValueKind {
    Float,
    Vec2,
    Vec3,
    Color,
}

const fn value_kind(v: WireValueKind) -> hsd::attributes::shader::value::ValueKind {
    use hsd::attributes::shader::value::ValueKind;
    match v {
        WireValueKind::Float => ValueKind::Float,
        WireValueKind::Vec2 => ValueKind::Vec2,
        WireValueKind::Vec3 => ValueKind::Vec3,
        WireValueKind::Color => ValueKind::Color,
    }
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(tag = "tag", content = "val", rename_all = "kebab-case")]
enum WireNode {
    Uv,
    WorldNormal,
    WorldPosition,
    VertexColor,
    LocalPosition,
    LocalNormal,
    Time,
    InstanceRandom,
    ObjectPosition,
    ObjectScale,
    ViewDirection,
    ScreenUv,

    Add(WireBinaryOp),
    Sub(WireBinaryOp),
    Mul(WireBinaryOp),
    Div(WireBinaryOp),
    Modulo(WireBinaryOp),
    Min(WireBinaryOp),
    Max(WireBinaryOp),
    Dot(WireBinaryOp),
    Cross(WireBinaryOp),
    Distance(WireBinaryOp),
    Pow(WirePowOp),
    Atan2(WireAtan2Op),
    Lerp(WireLerpOp),
    Clamp(WireClampOp),
    Step(WireStepOp),
    Smoothstep(WireSmoothstepOp),
    Remap(WireRemapOp),
    Select(WireSelectOp),

    Sin(WirePort),
    Cos(WirePort),
    OneMinus(WirePort),
    Abs(WirePort),
    Floor(WirePort),
    Fract(WirePort),
    Saturate(WirePort),
    Sqrt(WirePort),
    Length(WirePort),
    Normalize(WirePort),
    TriangleWave(WirePort),
    Luminance(WirePort),

    Fresnel(WirePort),
    Noise(WirePort),
    TextureSample(WireTextureSampleOp),
    SceneColor(WirePort),

    Extract(WireExtractOp),
    Combine2(WireCombine2Op),
    Combine3(WireCombine3Op),
    Combine4(WireCombine4Op),
    Convert(WireConvertOp),

    PolarCoords(WirePolarCoordsOp),
    RotateUv(WireRotateUvOp),
}

#[expect(clippy::too_many_lines, reason = "a 1:1 correspondence, not logic")]
const fn node(value: WireNode) -> Node {
    match value {
        WireNode::Uv => Node::Uv,
        WireNode::WorldNormal => Node::WorldNormal,
        WireNode::WorldPosition => Node::WorldPosition,
        WireNode::VertexColor => Node::VertexColor,
        WireNode::LocalPosition => Node::LocalPosition,
        WireNode::LocalNormal => Node::LocalNormal,
        WireNode::Time => Node::Time,
        WireNode::InstanceRandom => Node::InstanceRandom,
        WireNode::ObjectPosition => Node::ObjectPosition,
        WireNode::ObjectScale => Node::ObjectScale,
        WireNode::ViewDirection => Node::ViewDirection,
        WireNode::ScreenUv => Node::ScreenUv,

        WireNode::Add(op) => Node::Add {
            a: port(op.a),
            b: port(op.b),
        },
        WireNode::Sub(op) => Node::Sub {
            a: port(op.a),
            b: port(op.b),
        },
        WireNode::Mul(op) => Node::Mul {
            a: port(op.a),
            b: port(op.b),
        },
        WireNode::Div(op) => Node::Div {
            a: port(op.a),
            b: port(op.b),
        },
        WireNode::Modulo(op) => Node::Modulo {
            a: port(op.a),
            b: port(op.b),
        },
        WireNode::Min(op) => Node::Min {
            a: port(op.a),
            b: port(op.b),
        },
        WireNode::Max(op) => Node::Max {
            a: port(op.a),
            b: port(op.b),
        },
        WireNode::Dot(op) => Node::Dot {
            a: port(op.a),
            b: port(op.b),
        },
        WireNode::Cross(op) => Node::Cross {
            a: port(op.a),
            b: port(op.b),
        },
        WireNode::Distance(op) => Node::Distance {
            a: port(op.a),
            b: port(op.b),
        },
        WireNode::Pow(op) => Node::Pow {
            x: port(op.x),
            y: port(op.y),
        },
        WireNode::Atan2(op) => Node::Atan2 {
            y: port(op.y),
            x: port(op.x),
        },
        WireNode::Lerp(op) => Node::Lerp {
            a: port(op.a),
            b: port(op.b),
            t: port(op.t),
        },
        WireNode::Clamp(op) => Node::Clamp {
            x:    port(op.x),
            low:  port(op.low),
            high: port(op.high),
        },
        WireNode::Step(op) => Node::Step {
            edge: port(op.edge),
            x:    port(op.x),
        },
        WireNode::Smoothstep(op) => Node::Smoothstep {
            low:  port(op.low),
            high: port(op.high),
            x:    port(op.x),
        },
        WireNode::Remap(op) => Node::Remap {
            x:         port(op.x),
            from_low:  port(op.from_low),
            from_high: port(op.from_high),
            to_low:    port(op.to_low),
            to_high:   port(op.to_high),
        },
        WireNode::Select(op) => Node::Select {
            cond: port(op.cond),
            a:    port(op.a),
            b:    port(op.b),
        },

        WireNode::Sin(x) => Node::Sin { x: port(x) },
        WireNode::Cos(x) => Node::Cos { x: port(x) },
        WireNode::OneMinus(x) => Node::OneMinus { x: port(x) },
        WireNode::Abs(x) => Node::Abs { x: port(x) },
        WireNode::Floor(x) => Node::Floor { x: port(x) },
        WireNode::Fract(x) => Node::Fract { x: port(x) },
        WireNode::Saturate(x) => Node::Saturate { x: port(x) },
        WireNode::Sqrt(x) => Node::Sqrt { x: port(x) },
        WireNode::Length(v) => Node::Length { v: port(v) },
        WireNode::Normalize(v) => Node::Normalize { v: port(v) },
        WireNode::TriangleWave(x) => Node::TriangleWave { x: port(x) },
        WireNode::Luminance(color) => Node::Luminance { color: port(color) },
        WireNode::Fresnel(power) => Node::Fresnel { power: port(power) },
        WireNode::Noise(uv) => Node::Noise { uv: port(uv) },
        WireNode::SceneColor(uv) => Node::SceneColor { uv: port(uv) },
        WireNode::TextureSample(op) => Node::TextureSample {
            uv:   port(op.uv),
            slot: op.slot,
        },

        WireNode::Extract(op) => Node::Extract {
            v:       port(op.v),
            channel: op.channel,
        },
        WireNode::Combine2(op) => Node::Combine2 {
            x: port(op.x),
            y: port(op.y),
        },
        WireNode::Combine3(op) => Node::Combine3 {
            x: port(op.x),
            y: port(op.y),
            z: port(op.z),
        },
        WireNode::Combine4(op) => Node::Combine4 {
            x: port(op.x),
            y: port(op.y),
            z: port(op.z),
            w: port(op.w),
        },
        WireNode::Convert(op) => Node::Convert {
            v:  port(op.v),
            to: value_kind(op.to),
        },

        WireNode::PolarCoords(op) => Node::PolarCoords {
            uv:     port(op.uv),
            center: port(op.center),
        },
        WireNode::RotateUv(op) => Node::RotateUv {
            uv:      port(op.uv),
            center:  port(op.center),
            radians: port(op.radians),
        },
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireLitOutput {
    base_color:            Option<WirePort>,
    emissive:              Option<WirePort>,
    metallic:              Option<WirePort>,
    roughness:             Option<WirePort>,
    normal:                Option<WirePort>,
    alpha:                 Option<WirePort>,
    alpha_clip_threshold:  Option<WirePort>,
    specular_transmission: Option<WirePort>,
    diffuse_transmission:  Option<WirePort>,
    thickness:             Option<WirePort>,
    ior:                   Option<WirePort>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireUnlitOutput {
    color:                WirePort,
    alpha_clip_threshold: Option<WirePort>,
}

#[derive(Deserialize)]
#[serde(tag = "tag", content = "val", rename_all = "kebab-case")]
enum WireSurfaceOutput {
    Lit(WireLitOutput),
    Unlit(WireUnlitOutput),
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum WireBlendMode {
    Opaque,
    Blend,
    Add,
    Multiply,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum WireCullMode {
    Back,
    Front,
    None,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireSurfaceGraph {
    nodes:        Vec<WireNode>,
    output:       WireSurfaceOutput,
    blend:        WireBlendMode,
    cull:         WireCullMode,
    cast_shadows: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireDisplacementGraph {
    nodes:                 Vec<WireNode>,
    position_offset:       Option<WirePort>,
    normal_override:       Option<WirePort>,
    world_position_offset: Option<WirePort>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireShaderGraph {
    public_inputs: Vec<WireGraphValue>,
    surface:       WireSurfaceGraph,
    displacement:  Option<WireDisplacementGraph>,
}

fn surface(value: WireSurfaceGraph) -> SurfaceGraph {
    SurfaceGraph {
        nodes:        value.nodes.into_iter().map(node).collect(),
        output:       match value.output {
            WireSurfaceOutput::Lit(out) => SurfaceOutput::Lit(LitOutput {
                base_color:            out.base_color.map(port),
                emissive:              out.emissive.map(port),
                metallic:              out.metallic.map(port),
                roughness:             out.roughness.map(port),
                normal:                out.normal.map(port),
                alpha:                 out.alpha.map(port),
                alpha_clip_threshold:  out.alpha_clip_threshold.map(port),
                specular_transmission: out.specular_transmission.map(port),
                diffuse_transmission:  out.diffuse_transmission.map(port),
                thickness:             out.thickness.map(port),
                ior:                   out.ior.map(port),
            }),
            WireSurfaceOutput::Unlit(out) => SurfaceOutput::Unlit(UnlitOutput {
                color:                port(out.color),
                alpha_clip_threshold: out.alpha_clip_threshold.map(port),
            }),
        },
        blend:        match value.blend {
            WireBlendMode::Opaque => BlendMode::Opaque,
            WireBlendMode::Blend => BlendMode::Blend,
            WireBlendMode::Add => BlendMode::Add,
            WireBlendMode::Multiply => BlendMode::Multiply,
        },
        cull:         match value.cull {
            WireCullMode::Back => CullMode::Back,
            WireCullMode::Front => CullMode::Front,
            WireCullMode::None => CullMode::None,
        },
        cast_shadows: value.cast_shadows,
    }
}

fn displacement(value: WireDisplacementGraph) -> DisplacementGraph {
    DisplacementGraph {
        nodes:                 value.nodes.into_iter().map(node).collect(),
        position_offset:       value.position_offset.map(port),
        normal_override:       value.normal_override.map(port),
        world_position_offset: value.world_position_offset.map(port),
    }
}

fn graph(value: WireShaderGraph) -> ShaderGraph {
    ShaderGraph {
        public_inputs: value.public_inputs.into_iter().map(graph_value).collect(),
        surface:       surface(value.surface),
        displacement:  value.displacement.map(displacement),
    }
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "setGraph")]
    pub fn set_graph(
        &self,
        document: &DocumentHandle,
        prim: JsValue,
        layer: String,
        value: JsValue,
    ) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        let doc = document.rep();
        let parsed = (|| -> Result<_, ScriptError> {
            let prim = convert::prim_id(prim)?;
            let graph = if value.is_undefined() {
                None
            } else {
                let wire: WireShaderGraph = from_js(value)?;
                Some(graph(wire))
            };
            Ok((prim, graph))
        })();
        let (prim, built) = match parsed {
            Ok(v) => v,
            Err(err) => return js_sys::Promise::reject(&convert::raise(err)),
        };
        future_to_promise(async move {
            let host = host.borrow();
            shading::set_graph(&host, doc, prim, convert::layer(&layer), built)
                .await
                .map(|()| JsValue::UNDEFINED)
                .map_err(convert::raise)
        })
    }

    pub fn overrides(&self, document: &DocumentHandle, prim: JsValue) -> Result<JsValue, JsValue> {
        let prim = convert::prim_id(prim).map_err(convert::into_trap)?;
        let overrides = shading::overrides(&self.host.borrow(), document.rep(), prim)
            .map_err(convert::into_trap)?;
        Ok(overrides
            .into_iter()
            .map(|(index, value)| {
                js_sys::Array::of2(&JsValue::from(index), &to_js(&wire_graph_value(value)))
            })
            .collect::<js_sys::Array>()
            .into())
    }

    #[wasm_bindgen(js_name = "setOverrides")]
    pub fn set_overrides(
        &self,
        document: &DocumentHandle,
        prim: JsValue,
        layer: String,
        values: JsValue,
    ) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        let doc = document.rep();
        let parsed = (|| -> Result<_, ScriptError> {
            let prim = convert::prim_id(prim)?;
            let values = js_sys::Array::from(&values)
                .iter()
                .map(|entry| {
                    let pair = js_sys::Array::from(&entry);
                    let index = pair
                        .get(0)
                        .as_f64()
                        .ok_or_else(|| ScriptError::invalid("an override index is a number"))?
                        as u16;
                    let value: WireGraphValue = from_js(pair.get(1))?;
                    Ok((index, graph_value(value)))
                })
                .collect::<Result<Vec<_>, ScriptError>>()?;
            Ok((prim, values))
        })();
        let (prim, values) = match parsed {
            Ok(v) => v,
            Err(err) => return js_sys::Promise::reject(&convert::raise(err)),
        };
        future_to_promise(async move {
            let host = host.borrow();
            shading::set_overrides(&host, doc, prim, convert::layer(&layer), values)
                .await
                .map(|()| JsValue::UNDEFINED)
                .map_err(convert::raise)
        })
    }
}
