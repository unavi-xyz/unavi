mod arithmetic;
mod builtin;
mod channel;
mod derived;
mod leaf;
mod sample;

use std::fmt::{
    Result,
    Write,
};

use hsd::schema::shader::{
    node::{
        Node,
        Port,
    },
    value::{
        GraphValue,
        ValueKind,
    },
};

pub(super) const fn wgsl_type(kind: ValueKind) -> &'static str {
    match kind {
        ValueKind::Float => "f32",
        ValueKind::Vec2 => "vec2<f32>",
        ValueKind::Vec3 => "vec3<f32>",
        ValueKind::Color => "vec4<f32>",
    }
}

/// `{:?}` rather than `{}`: `f32`'s `Debug` always prints a decimal point
/// (`2.0`, not `2`), which a bare integer is not in WGSL.
fn literal(out: &mut String, value: GraphValue) -> Result {
    match value {
        GraphValue::Float(v) => write!(out, "{v:?}"),
        GraphValue::Vec2([x, y]) => write!(out, "vec2<f32>({x:?}, {y:?})"),
        GraphValue::Vec3([x, y, z]) => write!(out, "vec3<f32>({x:?}, {y:?}, {z:?})"),
        GraphValue::Color([r, g, b, a]) => write!(out, "vec4<f32>({r:?}, {g:?}, {b:?}, {a:?})"),
    }
}

/// A public input is stored as a full `vec4` slot; a reference swizzles down
/// to the components its declared kind uses.
pub(super) fn port_expr(out: &mut String, public_inputs: &[GraphValue], port: Port) -> Result {
    match port {
        Port::Const(value) => literal(out, value)?,
        Port::Input(index) => {
            let kind = public_inputs[usize::from(index)].kind();
            write!(out, "params.inputs[{index}]")?;
            match kind {
                ValueKind::Float => out.push_str(".x"),
                ValueKind::Vec2 => out.push_str(".xy"),
                ValueKind::Vec3 => out.push_str(".xyz"),
                ValueKind::Color => {}
            }
        }
        Port::Node(index) => {
            write!(out, "n{index}")?;
        }
    }
    Ok(())
}

/// The kind a port carries, for nodes whose generated expression depends on
/// it. Total: only a validated graph reaches codegen, and validation already
/// rejected every out-of-range reference.
fn port_kind(public_inputs: &[GraphValue], kinds: &[ValueKind], port: Port) -> ValueKind {
    match port {
        Port::Const(value) => value.kind(),
        Port::Input(index) => public_inputs[usize::from(index)].kind(),
        Port::Node(index) => kinds[usize::from(index)],
    }
}

fn zero_literal(out: &mut String, kind: ValueKind) {
    match kind {
        ValueKind::Float => out.push_str("0.0"),
        ValueKind::Vec2 => out.push_str("vec2<f32>(0.0)"),
        ValueKind::Vec3 => out.push_str("vec3<f32>(0.0)"),
        ValueKind::Color => out.push_str("vec4<f32>(0.0)"),
    }
}

/// The per-node WGSL expression: one exhaustive match dispatching each
/// variant to the family that owns it, so adding a node makes the compiler
/// name this file.
pub(super) fn node_expr(
    out: &mut String,
    public_inputs: &[GraphValue],
    kinds: &[ValueKind],
    node: &Node,
) -> Result {
    match *node {
        Node::Add { .. } | Node::Sub { .. } | Node::Mul { .. } | Node::Div { .. } => {
            arithmetic::emit(out, public_inputs, node)
        }
        Node::Lerp { .. }
        | Node::Dot { .. }
        | Node::Sin { .. }
        | Node::Cos { .. }
        | Node::OneMinus { .. }
        | Node::Abs { .. }
        | Node::Floor { .. }
        | Node::Fract { .. }
        | Node::Saturate { .. }
        | Node::Sqrt { .. }
        | Node::Pow { .. }
        | Node::Min { .. }
        | Node::Max { .. }
        | Node::Clamp { .. }
        | Node::Step { .. }
        | Node::Smoothstep { .. }
        | Node::Length { .. }
        | Node::Normalize { .. }
        | Node::Atan2 { .. }
        | Node::Modulo { .. }
        | Node::Distance { .. }
        | Node::Cross { .. } => builtin::emit(out, public_inputs, kinds, node),
        Node::Remap { .. }
        | Node::TriangleWave { .. }
        | Node::Luminance { .. }
        | Node::PolarCoords { .. }
        | Node::RotateUv { .. } => derived::emit(out, public_inputs, node),
        Node::Combine2 { .. }
        | Node::Combine3 { .. }
        | Node::Combine4 { .. }
        | Node::Extract { .. }
        | Node::Convert { .. } => channel::emit(out, public_inputs, kinds, node),
        Node::Uv
        | Node::WorldNormal
        | Node::WorldPosition
        | Node::VertexColor
        | Node::LocalPosition
        | Node::LocalNormal
        | Node::InstanceRandom
        | Node::ObjectPosition
        | Node::ObjectScale
        | Node::ViewDirection
        | Node::ScreenUv
        | Node::Time => {
            leaf::emit(out, node);
            Ok(())
        }
        Node::Fresnel { .. }
        | Node::Noise { .. }
        | Node::TextureSample { .. }
        | Node::SceneColor { .. }
        | Node::Select { .. } => sample::emit(out, public_inputs, node),
    }
}
