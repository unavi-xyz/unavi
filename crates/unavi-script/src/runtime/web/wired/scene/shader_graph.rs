//! Lifts a script-built shader graph off `jco`'s JS representation onto the
//! format's own types.
//!
//! The node vocabulary is stated twice — once as `hsd`'s `Node`, once as the
//! WIT `node` a script builds — and on this side the second copy is untyped
//! JS, so an unknown tag is an error rather than a compile failure the way the
//! native mirror gets.

use hsd::attributes::material_graph::{
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
    value::{
        GraphValue,
        ValueKind,
    },
};
use wasm_bindgen::JsValue;

use super::util::{
    js_to_vec3,
    obj_get,
    obj_get_bool,
    obj_get_f32,
    obj_get_i32,
    obj_get_string,
    obj_set,
    vec3_to_js,
};
use crate::runtime::shared::wired::scene::prim::{
    PrimColor,
    PrimGraphValue,
};

pub fn js_to_graph(v: &JsValue) -> Result<Option<ShaderGraph>, String> {
    if v.is_null() || v.is_undefined() {
        return Ok(None);
    }
    Ok(Some(ShaderGraph {
        public_inputs: js_array(&obj_get(v, "publicInputs"))
            .iter()
            .map(|value| js_to_graph_value(&value))
            .collect::<Result<_, _>>()?,
        surface:       surface(&obj_get(v, "surface"))?,
        displacement:  displacement(&obj_get(v, "displacement"))?,
    }))
}

pub const fn prim_value(value: GraphValue) -> PrimGraphValue {
    match value {
        GraphValue::Float(v) => PrimGraphValue::Float(v),
        GraphValue::Vec2(v) => PrimGraphValue::Vec2(v),
        GraphValue::Vec3(v) => PrimGraphValue::Vec3(v),
        GraphValue::Color([r, g, b, a]) => PrimGraphValue::Color(PrimColor { r, g, b, a }),
    }
}

pub const fn graph_value(value: PrimGraphValue) -> GraphValue {
    match value {
        PrimGraphValue::Float(v) => GraphValue::Float(v),
        PrimGraphValue::Vec2(v) => GraphValue::Vec2(v),
        PrimGraphValue::Vec3(v) => GraphValue::Vec3(v),
        PrimGraphValue::Color(c) => GraphValue::Color([c.r, c.g, c.b, c.a]),
    }
}

pub fn graph_value_to_js(value: GraphValue) -> JsValue {
    match value {
        GraphValue::Float(v) => variant("float", v.into()),
        GraphValue::Vec2([x, y]) => variant("vec2", vec2_to_js(x, y)),
        GraphValue::Vec3([x, y, z]) => variant("vec3", vec3_to_js(x, y, z)),
        GraphValue::Color([r, g, b, a]) => variant("color", color_to_js(r, g, b, a)),
    }
}

pub fn js_to_graph_value(v: &JsValue) -> Result<GraphValue, String> {
    let tag = obj_get_string(v, "tag").ok_or("a graph value needs a tag")?;
    let val = obj_get(v, "val");
    Ok(match tag.as_str() {
        "float" => GraphValue::Float(f32(&val)),
        "vec2" => GraphValue::Vec2([obj_f32(&val, "x"), obj_f32(&val, "y")]),
        "vec3" => GraphValue::Vec3(js_to_vec3(&val, [0.0; 3])),
        "color" => GraphValue::Color([
            obj_f32(&val, "r"),
            obj_f32(&val, "g"),
            obj_f32(&val, "b"),
            obj_f32(&val, "a"),
        ]),
        other => return Err(format!("unknown graph value '{other}'")),
    })
}

fn variant(tag: &str, val: JsValue) -> JsValue {
    let obj = js_sys::Object::new();
    obj_set(&obj, "tag", &tag.into());
    obj_set(&obj, "val", &val);
    obj.into()
}

fn vec2_to_js(x: f32, y: f32) -> JsValue {
    let obj = js_sys::Object::new();
    obj_set(&obj, "x", &x.into());
    obj_set(&obj, "y", &y.into());
    obj.into()
}

fn color_to_js(r: f32, g: f32, b: f32, a: f32) -> JsValue {
    let obj = js_sys::Object::new();
    obj_set(&obj, "r", &r.into());
    obj_set(&obj, "g", &g.into());
    obj_set(&obj, "b", &b.into());
    obj_set(&obj, "a", &a.into());
    obj.into()
}

fn js_array(v: &JsValue) -> js_sys::Array {
    if v.is_null() || v.is_undefined() {
        js_sys::Array::new()
    } else {
        js_sys::Array::from(v)
    }
}

fn f32(v: &JsValue) -> f32 {
    v.as_f64().unwrap_or_default() as f32
}

fn obj_f32(v: &JsValue, key: &str) -> f32 {
    obj_get_f32(v, key).unwrap_or_default()
}

fn nodes(v: &JsValue) -> Result<Vec<Node>, String> {
    js_array(v).iter().map(|n| node(&n)).collect()
}

fn surface(v: &JsValue) -> Result<SurfaceGraph, String> {
    if v.is_null() || v.is_undefined() {
        return Err("a shader graph needs a surface network".to_string());
    }
    Ok(SurfaceGraph {
        nodes:        nodes(&obj_get(v, "nodes"))?,
        output:       output(&obj_get(v, "output"))?,
        blend:        blend(&obj_get(v, "blend"))?,
        cull:         cull(&obj_get(v, "cull"))?,
        cast_shadows: obj_get_bool(v, "castShadows").unwrap_or_default(),
    })
}

fn displacement(v: &JsValue) -> Result<Option<DisplacementGraph>, String> {
    if v.is_null() || v.is_undefined() {
        return Ok(None);
    }
    Ok(Some(DisplacementGraph {
        nodes:                 nodes(&obj_get(v, "nodes"))?,
        position_offset:       opt_port(v, "positionOffset")?,
        normal_override:       opt_port(v, "normalOverride")?,
        world_position_offset: opt_port(v, "worldPositionOffset")?,
    }))
}

fn output(v: &JsValue) -> Result<SurfaceOutput, String> {
    let tag = obj_get_string(v, "tag").ok_or("a surface output needs a tag")?;
    let val = obj_get(v, "val");
    Ok(match tag.as_str() {
        "lit" => SurfaceOutput::Lit(LitOutput {
            base_color:            opt_port(&val, "baseColor")?,
            emissive:              opt_port(&val, "emissive")?,
            metallic:              opt_port(&val, "metallic")?,
            roughness:             opt_port(&val, "roughness")?,
            normal:                opt_port(&val, "normal")?,
            alpha:                 opt_port(&val, "alpha")?,
            alpha_clip_threshold:  opt_port(&val, "alphaClipThreshold")?,
            specular_transmission: opt_port(&val, "specularTransmission")?,
            diffuse_transmission:  opt_port(&val, "diffuseTransmission")?,
            thickness:             opt_port(&val, "thickness")?,
            ior:                   opt_port(&val, "ior")?,
        }),
        "unlit" => SurfaceOutput::Unlit(UnlitOutput {
            color:                port(&obj_get(&val, "color"))?,
            alpha_clip_threshold: opt_port(&val, "alphaClipThreshold")?,
        }),
        other => return Err(format!("unknown surface output '{other}'")),
    })
}

fn blend(v: &JsValue) -> Result<BlendMode, String> {
    Ok(match v.as_string().as_deref() {
        Some("opaque") => BlendMode::Opaque,
        Some("blend") => BlendMode::Blend,
        Some("add") => BlendMode::Add,
        Some("multiply") => BlendMode::Multiply,
        other => {
            return Err(format!(
                "unknown blend mode '{}'",
                other.unwrap_or_default()
            ));
        }
    })
}

fn cull(v: &JsValue) -> Result<CullMode, String> {
    Ok(match v.as_string().as_deref() {
        Some("back") => CullMode::Back,
        Some("front") => CullMode::Front,
        Some("none") => CullMode::None,
        other => return Err(format!("unknown cull mode '{}'", other.unwrap_or_default())),
    })
}

fn value_kind(v: &JsValue) -> Result<ValueKind, String> {
    Ok(match v.as_string().as_deref() {
        Some("float") => ValueKind::Float,
        Some("vec2") => ValueKind::Vec2,
        Some("vec3") => ValueKind::Vec3,
        Some("color") => ValueKind::Color,
        other => {
            return Err(format!(
                "unknown value kind '{}'",
                other.unwrap_or_default()
            ));
        }
    })
}

fn opt_port(obj: &JsValue, key: &str) -> Result<Option<Port>, String> {
    let v = obj_get(obj, key);
    if v.is_null() || v.is_undefined() {
        return Ok(None);
    }
    port(&v).map(Some)
}

fn port(v: &JsValue) -> Result<Port, String> {
    let tag = obj_get_string(v, "tag").ok_or("a port needs a tag")?;
    let val = obj_get(v, "val");
    Ok(match tag.as_str() {
        "const" => Port::Const(js_to_graph_value(&val)?),
        "input" => Port::Input(index(&val)?),
        "node" => Port::Node(index(&val)?),
        other => return Err(format!("unknown port '{other}'")),
    })
}

fn index(v: &JsValue) -> Result<u16, String> {
    v.as_f64()
        .and_then(|v| u16::try_from(v as i64).ok())
        .ok_or_else(|| "a port index is a u16".to_string())
}

fn u8_field(v: &JsValue, key: &str) -> Result<u8, String> {
    obj_get_i32(v, key)
        .and_then(|v| u8::try_from(v).ok())
        .ok_or_else(|| format!("'{key}' is a u8"))
}

fn binary(v: &JsValue) -> Result<(Port, Port), String> {
    Ok((port(&obj_get(v, "a"))?, port(&obj_get(v, "b"))?))
}

/// Kept as one table rather than split by family the way `hsd`'s validation
/// and `bevy-hsd`'s codegen are: those group by family because each family
/// carries different rules, where this carries none.
#[expect(clippy::too_many_lines, reason = "a 1:1 correspondence, not logic")]
fn node(v: &JsValue) -> Result<Node, String> {
    let tag = obj_get_string(v, "tag").ok_or("a node needs a tag")?;
    let val = obj_get(v, "val");
    Ok(match tag.as_str() {
        "uv" => Node::Uv,
        "world-normal" => Node::WorldNormal,
        "world-position" => Node::WorldPosition,
        "vertex-color" => Node::VertexColor,
        "local-position" => Node::LocalPosition,
        "local-normal" => Node::LocalNormal,
        "time" => Node::Time,
        "instance-random" => Node::InstanceRandom,
        "object-position" => Node::ObjectPosition,
        "object-scale" => Node::ObjectScale,
        "view-direction" => Node::ViewDirection,
        "screen-uv" => Node::ScreenUv,

        "add" => {
            let (a, b) = binary(&val)?;
            Node::Add { a, b }
        }
        "sub" => {
            let (a, b) = binary(&val)?;
            Node::Sub { a, b }
        }
        "mul" => {
            let (a, b) = binary(&val)?;
            Node::Mul { a, b }
        }
        "div" => {
            let (a, b) = binary(&val)?;
            Node::Div { a, b }
        }
        "modulo" => {
            let (a, b) = binary(&val)?;
            Node::Modulo { a, b }
        }
        "min" => {
            let (a, b) = binary(&val)?;
            Node::Min { a, b }
        }
        "max" => {
            let (a, b) = binary(&val)?;
            Node::Max { a, b }
        }
        "dot" => {
            let (a, b) = binary(&val)?;
            Node::Dot { a, b }
        }
        "cross" => {
            let (a, b) = binary(&val)?;
            Node::Cross { a, b }
        }
        "distance" => {
            let (a, b) = binary(&val)?;
            Node::Distance { a, b }
        }
        "pow" => Node::Pow {
            x: port(&obj_get(&val, "x"))?,
            y: port(&obj_get(&val, "y"))?,
        },
        "atan2" => Node::Atan2 {
            y: port(&obj_get(&val, "y"))?,
            x: port(&obj_get(&val, "x"))?,
        },
        "lerp" => Node::Lerp {
            a: port(&obj_get(&val, "a"))?,
            b: port(&obj_get(&val, "b"))?,
            t: port(&obj_get(&val, "t"))?,
        },
        "clamp" => Node::Clamp {
            x:    port(&obj_get(&val, "x"))?,
            low:  port(&obj_get(&val, "low"))?,
            high: port(&obj_get(&val, "high"))?,
        },
        "step" => Node::Step {
            edge: port(&obj_get(&val, "edge"))?,
            x:    port(&obj_get(&val, "x"))?,
        },
        "smoothstep" => Node::Smoothstep {
            low:  port(&obj_get(&val, "low"))?,
            high: port(&obj_get(&val, "high"))?,
            x:    port(&obj_get(&val, "x"))?,
        },
        "remap" => Node::Remap {
            x:         port(&obj_get(&val, "x"))?,
            from_low:  port(&obj_get(&val, "fromLow"))?,
            from_high: port(&obj_get(&val, "fromHigh"))?,
            to_low:    port(&obj_get(&val, "toLow"))?,
            to_high:   port(&obj_get(&val, "toHigh"))?,
        },
        "select" => Node::Select {
            cond: port(&obj_get(&val, "cond"))?,
            a:    port(&obj_get(&val, "a"))?,
            b:    port(&obj_get(&val, "b"))?,
        },

        "sin" => Node::Sin { x: port(&val)? },
        "cos" => Node::Cos { x: port(&val)? },
        "one-minus" => Node::OneMinus { x: port(&val)? },
        "abs" => Node::Abs { x: port(&val)? },
        "floor" => Node::Floor { x: port(&val)? },
        "fract" => Node::Fract { x: port(&val)? },
        "saturate" => Node::Saturate { x: port(&val)? },
        "sqrt" => Node::Sqrt { x: port(&val)? },
        "length" => Node::Length { v: port(&val)? },
        "normalize" => Node::Normalize { v: port(&val)? },
        "triangle-wave" => Node::TriangleWave { x: port(&val)? },
        "luminance" => Node::Luminance { color: port(&val)? },
        "fresnel" => Node::Fresnel { power: port(&val)? },
        "noise" => Node::Noise { uv: port(&val)? },
        "scene-color" => Node::SceneColor { uv: port(&val)? },
        "texture-sample" => Node::TextureSample {
            uv:   port(&obj_get(&val, "uv"))?,
            slot: u8_field(&val, "slot")?,
        },

        "extract" => Node::Extract {
            v:       port(&obj_get(&val, "v"))?,
            channel: u8_field(&val, "channel")?,
        },
        "combine2" => Node::Combine2 {
            x: port(&obj_get(&val, "x"))?,
            y: port(&obj_get(&val, "y"))?,
        },
        "combine3" => Node::Combine3 {
            x: port(&obj_get(&val, "x"))?,
            y: port(&obj_get(&val, "y"))?,
            z: port(&obj_get(&val, "z"))?,
        },
        "combine4" => Node::Combine4 {
            x: port(&obj_get(&val, "x"))?,
            y: port(&obj_get(&val, "y"))?,
            z: port(&obj_get(&val, "z"))?,
            w: port(&obj_get(&val, "w"))?,
        },
        "convert" => Node::Convert {
            v:  port(&obj_get(&val, "v"))?,
            to: value_kind(&obj_get(&val, "to"))?,
        },

        "polar-coords" => Node::PolarCoords {
            uv:     port(&obj_get(&val, "uv"))?,
            center: port(&obj_get(&val, "center"))?,
        },
        "rotate-uv" => Node::RotateUv {
            uv:      port(&obj_get(&val, "uv"))?,
            center:  port(&obj_get(&val, "center"))?,
            radians: port(&obj_get(&val, "radians"))?,
        },

        other => return Err(format!("unknown node '{other}'")),
    })
}
