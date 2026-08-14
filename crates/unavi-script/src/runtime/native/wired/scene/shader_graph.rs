//! Lowers a script-built shader graph onto the format's own types.
//!
//! The node vocabulary is stated twice — once as `hsd`'s `Node`, once as the
//! WIT `node` a script builds — and nothing generates one from the other, so
//! this file is where they are held together. Both matches are exhaustive with
//! no wildcard, so a kind added to either side fails to compile until it is
//! added here too.

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

use super::bindings::wired::scene::types as wit;

pub fn graph(value: wit::ShaderGraph) -> ShaderGraph {
    ShaderGraph {
        public_inputs: value.public_inputs.into_iter().map(graph_value).collect(),
        surface:       surface(value.surface),
        displacement:  value.displacement.map(displacement),
    }
}

fn surface(value: wit::SurfaceGraph) -> SurfaceGraph {
    SurfaceGraph {
        nodes:        value.nodes.into_iter().map(node).collect(),
        output:       match value.output {
            wit::SurfaceOutput::Lit(out) => SurfaceOutput::Lit(LitOutput {
                base_color:           out.base_color.map(port),
                emissive:             out.emissive.map(port),
                metallic:             out.metallic.map(port),
                roughness:            out.roughness.map(port),
                normal:               out.normal.map(port),
                alpha:                out.alpha.map(port),
                alpha_clip_threshold: out.alpha_clip_threshold.map(port),
            }),
            wit::SurfaceOutput::Unlit(out) => SurfaceOutput::Unlit(UnlitOutput {
                color:                port(out.color),
                alpha_clip_threshold: out.alpha_clip_threshold.map(port),
            }),
        },
        blend:        match value.blend {
            wit::BlendMode::Opaque => BlendMode::Opaque,
            wit::BlendMode::Blend => BlendMode::Blend,
            wit::BlendMode::Add => BlendMode::Add,
            wit::BlendMode::Multiply => BlendMode::Multiply,
        },
        cull:         match value.cull {
            wit::CullMode::Back => CullMode::Back,
            wit::CullMode::Front => CullMode::Front,
            wit::CullMode::None => CullMode::None,
        },
        cast_shadows: value.cast_shadows,
    }
}

fn displacement(value: wit::DisplacementGraph) -> DisplacementGraph {
    DisplacementGraph {
        nodes:                 value.nodes.into_iter().map(node).collect(),
        position_offset:       value.position_offset.map(port),
        normal_override:       value.normal_override.map(port),
        world_position_offset: value.world_position_offset.map(port),
    }
}

const fn graph_value(value: wit::GraphValue) -> GraphValue {
    match value {
        wit::GraphValue::Float(v) => GraphValue::Float(v),
        wit::GraphValue::Vec2(v) => GraphValue::Vec2([v.x, v.y]),
        wit::GraphValue::Vec3(v) => GraphValue::Vec3([v.x, v.y, v.z]),
        wit::GraphValue::Color(c) => GraphValue::Color([c.r, c.g, c.b, c.a]),
    }
}

const fn value_kind(value: wit::ValueKind) -> ValueKind {
    match value {
        wit::ValueKind::Float => ValueKind::Float,
        wit::ValueKind::Vec2 => ValueKind::Vec2,
        wit::ValueKind::Vec3 => ValueKind::Vec3,
        wit::ValueKind::Color => ValueKind::Color,
    }
}

const fn port(value: wit::Port) -> Port {
    match value {
        wit::Port::Const(v) => Port::Const(graph_value(v)),
        wit::Port::Input(index) => Port::Input(index),
        wit::Port::Node(index) => Port::Node(index),
    }
}

/// Kept as one table rather than split by family the way `hsd`'s validation
/// and `bevy-hsd`'s codegen are: those group by family because each family
/// carries different rules, where this carries none. Every arm says only which
/// kind it is, and the length is the vocabulary's.
#[expect(clippy::too_many_lines, reason = "a 1:1 correspondence, not logic")]
const fn node(value: wit::Node) -> Node {
    match value {
        wit::Node::Uv => Node::Uv,
        wit::Node::WorldNormal => Node::WorldNormal,
        wit::Node::WorldPosition => Node::WorldPosition,
        wit::Node::VertexColor => Node::VertexColor,
        wit::Node::LocalPosition => Node::LocalPosition,
        wit::Node::LocalNormal => Node::LocalNormal,
        wit::Node::Time => Node::Time,
        wit::Node::InstanceRandom => Node::InstanceRandom,
        wit::Node::ObjectPosition => Node::ObjectPosition,
        wit::Node::ObjectScale => Node::ObjectScale,
        wit::Node::ViewDirection => Node::ViewDirection,

        wit::Node::Add(op) => Node::Add {
            a: port(op.a),
            b: port(op.b),
        },
        wit::Node::Sub(op) => Node::Sub {
            a: port(op.a),
            b: port(op.b),
        },
        wit::Node::Mul(op) => Node::Mul {
            a: port(op.a),
            b: port(op.b),
        },
        wit::Node::Div(op) => Node::Div {
            a: port(op.a),
            b: port(op.b),
        },
        wit::Node::Modulo(op) => Node::Modulo {
            a: port(op.a),
            b: port(op.b),
        },
        wit::Node::Min(op) => Node::Min {
            a: port(op.a),
            b: port(op.b),
        },
        wit::Node::Max(op) => Node::Max {
            a: port(op.a),
            b: port(op.b),
        },
        wit::Node::Dot(op) => Node::Dot {
            a: port(op.a),
            b: port(op.b),
        },
        wit::Node::Cross(op) => Node::Cross {
            a: port(op.a),
            b: port(op.b),
        },
        wit::Node::Distance(op) => Node::Distance {
            a: port(op.a),
            b: port(op.b),
        },
        wit::Node::Pow(op) => Node::Pow {
            x: port(op.x),
            y: port(op.y),
        },
        wit::Node::Atan2(op) => Node::Atan2 {
            y: port(op.y),
            x: port(op.x),
        },
        wit::Node::Lerp(op) => Node::Lerp {
            a: port(op.a),
            b: port(op.b),
            t: port(op.t),
        },
        wit::Node::Clamp(op) => Node::Clamp {
            x:    port(op.x),
            low:  port(op.low),
            high: port(op.high),
        },
        wit::Node::Step(op) => Node::Step {
            edge: port(op.edge),
            x:    port(op.x),
        },
        wit::Node::Smoothstep(op) => Node::Smoothstep {
            low:  port(op.low),
            high: port(op.high),
            x:    port(op.x),
        },
        wit::Node::Remap(op) => Node::Remap {
            x:         port(op.x),
            from_low:  port(op.from_low),
            from_high: port(op.from_high),
            to_low:    port(op.to_low),
            to_high:   port(op.to_high),
        },
        wit::Node::Select(op) => Node::Select {
            cond: port(op.cond),
            a:    port(op.a),
            b:    port(op.b),
        },

        wit::Node::Sin(x) => Node::Sin { x: port(x) },
        wit::Node::Cos(x) => Node::Cos { x: port(x) },
        wit::Node::OneMinus(x) => Node::OneMinus { x: port(x) },
        wit::Node::Abs(x) => Node::Abs { x: port(x) },
        wit::Node::Floor(x) => Node::Floor { x: port(x) },
        wit::Node::Fract(x) => Node::Fract { x: port(x) },
        wit::Node::Saturate(x) => Node::Saturate { x: port(x) },
        wit::Node::Sqrt(x) => Node::Sqrt { x: port(x) },
        wit::Node::Length(v) => Node::Length { v: port(v) },
        wit::Node::Normalize(v) => Node::Normalize { v: port(v) },
        wit::Node::TriangleWave(x) => Node::TriangleWave { x: port(x) },
        wit::Node::Luminance(color) => Node::Luminance { color: port(color) },
        wit::Node::Fresnel(power) => Node::Fresnel { power: port(power) },
        wit::Node::Noise(uv) => Node::Noise { uv: port(uv) },
        wit::Node::TextureSample(op) => Node::TextureSample {
            uv:   port(op.uv),
            slot: op.slot,
        },

        wit::Node::Extract(op) => Node::Extract {
            v:       port(op.v),
            channel: op.channel,
        },
        wit::Node::Combine2(op) => Node::Combine2 {
            x: port(op.x),
            y: port(op.y),
        },
        wit::Node::Combine3(op) => Node::Combine3 {
            x: port(op.x),
            y: port(op.y),
            z: port(op.z),
        },
        wit::Node::Combine4(op) => Node::Combine4 {
            x: port(op.x),
            y: port(op.y),
            z: port(op.z),
            w: port(op.w),
        },
        wit::Node::Convert(op) => Node::Convert {
            v:  port(op.v),
            to: value_kind(op.to),
        },

        wit::Node::PolarCoords(op) => Node::PolarCoords {
            uv:     port(op.uv),
            center: port(op.center),
        },
        wit::Node::RotateUv(op) => Node::RotateUv {
            uv:      port(op.uv),
            center:  port(op.center),
            radians: port(op.radians),
        },
    }
}

#[cfg(test)]
mod tests {
    use hsd::attributes::material_graph::validate::validate;

    use super::*;

    /// Appends nodes and hands back a port naming each, so a test states what
    /// it is building rather than counting indices.
    #[derive(Default)]
    struct Net {
        nodes: Vec<wit::Node>,
    }

    impl Net {
        fn push(&mut self, node: wit::Node) -> wit::Port {
            self.nodes.push(node);
            wit::Port::Node(
                u16::try_from(self.nodes.len() - 1).expect("a test net stays far under 65k nodes"),
            )
        }
    }

    const fn f(v: f32) -> wit::Port {
        wit::Port::Const(wit::GraphValue::Float(v))
    }

    fn v2(x: f32, y: f32) -> wit::Port {
        wit::Port::Const(wit::GraphValue::Vec2(wit::Vec2 { x, y }))
    }

    fn v3() -> wit::Port {
        wit::Port::Const(wit::GraphValue::Vec3(wit::Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }))
    }

    fn binary(a: wit::Port, b: wit::Port) -> wit::BinaryOp {
        wit::BinaryOp { a, b }
    }

    fn float_of(port: Port) -> f32 {
        match port {
            Port::Const(GraphValue::Float(v)) => v,
            other => panic!("expected a float constant, got {other:?}"),
        }
    }

    /// A whole graph rather than a kind at a time: conversion is only correct
    /// if what comes out the other side is a graph the format accepts, and a
    /// port lowered onto the wrong field shows up as a type error here.
    ///
    /// The terminal is a constant, because every node in a network is checked
    /// whether or not a terminal reaches it — so the graph does not have to be
    /// contorted into using each kind it covers.
    fn assert_surface_converts(net: Net) {
        let converted = graph(wit::ShaderGraph {
            public_inputs: vec![wit::GraphValue::Float(0.0)],
            surface:       wit::SurfaceGraph {
                nodes:        net.nodes,
                output:       wit::SurfaceOutput::Unlit(wit::UnlitOutput {
                    color:                wit::Port::Const(wit::GraphValue::Color(wit::Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    })),
                    alpha_clip_threshold: None,
                }),
                blend:        wit::BlendMode::Add,
                cull:         wit::CullMode::None,
                cast_shadows: false,
            },
            displacement:  None,
        });
        validate(&converted).expect("a converted graph is a valid graph");
    }

    #[test]
    fn the_surface_context_leaves_convert() {
        let mut net = Net::default();
        for leaf in [
            wit::Node::Uv,
            wit::Node::WorldNormal,
            wit::Node::WorldPosition,
            wit::Node::VertexColor,
            wit::Node::Time,
            wit::Node::InstanceRandom,
            wit::Node::ObjectPosition,
            wit::Node::ObjectScale,
            wit::Node::ViewDirection,
        ] {
            net.push(leaf);
        }
        assert_surface_converts(net);
    }

    #[test]
    fn the_math_kinds_convert() {
        let mut net = Net::default();
        let normal = net.push(wit::Node::WorldNormal);
        let position = net.push(wit::Node::WorldPosition);
        let time = net.push(wit::Node::Time);
        let random = net.push(wit::Node::InstanceRandom);
        let origin = net.push(wit::Node::ObjectPosition);
        let scale = net.push(wit::Node::ObjectScale);
        let view = net.push(wit::Node::ViewDirection);

        net.push(wit::Node::Add(binary(time, random)));
        net.push(wit::Node::Sub(binary(f(1.0), time)));
        net.push(wit::Node::Mul(binary(normal, time)));
        net.push(wit::Node::Div(binary(scale, f(2.0))));
        net.push(wit::Node::Modulo(binary(time, f(2.0))));
        net.push(wit::Node::Min(binary(time, f(1.0))));
        net.push(wit::Node::Max(binary(time, f(0.0))));
        net.push(wit::Node::Dot(binary(normal, view)));
        net.push(wit::Node::Cross(binary(normal, view)));
        net.push(wit::Node::Distance(binary(position, origin)));
        net.push(wit::Node::Pow(wit::PowOp { x: time, y: f(2.0) }));
        net.push(wit::Node::Atan2(wit::Atan2Op { y: time, x: f(1.0) }));
        net.push(wit::Node::Lerp(wit::LerpOp {
            a: f(0.0),
            b: f(1.0),
            t: random,
        }));
        net.push(wit::Node::Clamp(wit::ClampOp {
            x:    time,
            low:  f(0.0),
            high: f(1.0),
        }));
        net.push(wit::Node::Step(wit::StepOp {
            edge: f(0.5),
            x:    time,
        }));
        net.push(wit::Node::Smoothstep(wit::SmoothstepOp {
            low:  f(0.0),
            high: f(1.0),
            x:    time,
        }));
        net.push(wit::Node::Remap(wit::RemapOp {
            x:         time,
            from_low:  f(0.0),
            from_high: f(1.0),
            to_low:    f(-1.0),
            to_high:   f(1.0),
        }));
        net.push(wit::Node::Select(wit::SelectOp {
            cond: random,
            a:    f(0.0),
            b:    f(1.0),
        }));
        assert_surface_converts(net);
    }

    #[test]
    fn the_unary_channel_and_uv_kinds_convert() {
        let mut net = Net::default();
        let uv = net.push(wit::Node::Uv);
        let normal = net.push(wit::Node::WorldNormal);
        let color = net.push(wit::Node::VertexColor);
        let time = net.push(wit::Node::Time);
        let random = net.push(wit::Node::InstanceRandom);

        for unary in [
            wit::Node::Sin as fn(wit::Port) -> wit::Node,
            wit::Node::Cos,
            wit::Node::OneMinus,
            wit::Node::Abs,
            wit::Node::Floor,
            wit::Node::Fract,
            wit::Node::Saturate,
            wit::Node::Sqrt,
            wit::Node::TriangleWave,
            wit::Node::Fresnel,
        ] {
            net.push(unary(time));
        }
        net.push(wit::Node::Length(normal));
        net.push(wit::Node::Normalize(normal));
        net.push(wit::Node::Luminance(color));
        net.push(wit::Node::Noise(uv));
        net.push(wit::Node::TextureSample(wit::TextureSampleOp {
            uv,
            slot: 0,
        }));

        let x = net.push(wit::Node::Extract(wit::ExtractOp {
            v:       normal,
            channel: 0,
        }));
        net.push(wit::Node::Combine2(wit::Combine2Op { x, y: time }));
        net.push(wit::Node::Combine3(wit::Combine3Op {
            x,
            y: time,
            z: random,
        }));
        net.push(wit::Node::Convert(wit::ConvertOp {
            v:  normal,
            to: wit::ValueKind::Color,
        }));
        net.push(wit::Node::PolarCoords(wit::PolarCoordsOp {
            uv,
            center: v2(0.5, 0.5),
        }));
        net.push(wit::Node::RotateUv(wit::RotateUvOp {
            uv,
            center: v2(0.5, 0.5),
            radians: time,
        }));
        net.push(wit::Node::Combine4(wit::Combine4Op {
            x,
            y: time,
            z: random,
            w: f(1.0),
        }));
        assert_surface_converts(net);
    }

    #[test]
    fn the_displacement_only_kinds_convert_too() {
        let mut net = Net::default();
        let local = net.push(wit::Node::LocalPosition);
        let normal = net.push(wit::Node::LocalNormal);
        let time = net.push(wit::Node::Time);
        let wave = net.push(wit::Node::Sin(time));
        let offset = net.push(wit::Node::Mul(binary(normal, wave)));

        let converted = graph(wit::ShaderGraph {
            public_inputs: Vec::new(),
            surface:       wit::SurfaceGraph {
                nodes:        Vec::new(),
                output:       wit::SurfaceOutput::Lit(wit::LitOutput {
                    base_color:           None,
                    emissive:             None,
                    metallic:             None,
                    roughness:            None,
                    normal:               None,
                    alpha:                None,
                    alpha_clip_threshold: None,
                }),
                blend:        wit::BlendMode::Opaque,
                cull:         wit::CullMode::Back,
                cast_shadows: true,
            },
            displacement:  Some(wit::DisplacementGraph {
                nodes:                 net.nodes,
                position_offset:       Some(offset),
                normal_override:       Some(local),
                world_position_offset: Some(v3()),
            }),
        });

        validate(&converted).expect("a converted displacement graph is valid");
    }

    /// These four have ports of one kind whose order carries meaning, so a
    /// swapped pair still type-checks and still validates. Only naming them
    /// catches it.
    #[test]
    fn ports_whose_order_matters_keep_it() {
        let Node::Atan2 { y, x } = node(wit::Node::Atan2(wit::Atan2Op {
            y: f(1.0),
            x: f(2.0),
        })) else {
            panic!("expected an atan2")
        };
        assert_eq!((float_of(y), float_of(x)), (1.0, 2.0));

        let Node::Step { edge, x } = node(wit::Node::Step(wit::StepOp {
            edge: f(1.0),
            x:    f(2.0),
        })) else {
            panic!("expected a step")
        };
        assert_eq!((float_of(edge), float_of(x)), (1.0, 2.0));

        let Node::Smoothstep { low, high, x } = node(wit::Node::Smoothstep(wit::SmoothstepOp {
            low:  f(1.0),
            high: f(2.0),
            x:    f(3.0),
        })) else {
            panic!("expected a smoothstep")
        };
        assert_eq!((float_of(low), float_of(high), float_of(x)), (1.0, 2.0, 3.0));

        let Node::Remap {
            x,
            from_low,
            from_high,
            to_low,
            to_high,
        } = node(wit::Node::Remap(wit::RemapOp {
            x:         f(1.0),
            from_low:  f(2.0),
            from_high: f(3.0),
            to_low:    f(4.0),
            to_high:   f(5.0),
        }))
        else {
            panic!("expected a remap")
        };
        assert_eq!(
            (
                float_of(x),
                float_of(from_low),
                float_of(from_high),
                float_of(to_low),
                float_of(to_high),
            ),
            (1.0, 2.0, 3.0, 4.0, 5.0)
        );
    }
}
