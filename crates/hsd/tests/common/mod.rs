// Compiled into every integration-test binary in this crate, each of which
// only uses a subset of these helpers.
#![expect(dead_code)]

//! Typed graph builders. Tests construct [`ShaderGraph`] values structurally
//! instead of parsing `.hss` authoring strings: a graph built from a string
//! would exercise RON deserialization, which is `hsd-cli`'s job and not what
//! this suite tests.

use hsd::attributes::material_graph::{
    ShaderGraph,
    graph::{
        DisplacementGraph,
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

#[must_use]
pub const fn const_f(v: f32) -> Port {
    Port::Const(GraphValue::Float(v))
}

#[must_use]
pub const fn const_v2(v: [f32; 2]) -> Port {
    Port::Const(GraphValue::Vec2(v))
}

#[must_use]
pub const fn const_v3(v: [f32; 3]) -> Port {
    Port::Const(GraphValue::Vec3(v))
}

#[must_use]
pub const fn const_color(v: [f32; 4]) -> Port {
    Port::Const(GraphValue::Color(v))
}

#[must_use]
pub const fn node(i: u16) -> Port {
    Port::Node(i)
}

#[must_use]
pub const fn input(i: u16) -> Port {
    Port::Input(i)
}

/// An unlit surface output with the given color and no clip threshold.
#[must_use]
pub const fn unlit(color: Port) -> SurfaceOutput {
    SurfaceOutput::Unlit(UnlitOutput {
        color,
        alpha_clip_threshold: None,
    })
}

/// A graph whose surface network holds `nodes` and has a default unlit output.
#[must_use]
pub fn graph(nodes: Vec<Node>) -> ShaderGraph {
    ShaderGraph {
        surface: SurfaceGraph {
            nodes,
            output: SurfaceOutput::Unlit(UnlitOutput::default()),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// A graph whose surface network holds `nodes` and the given output.
#[must_use]
pub fn graph_with_output(nodes: Vec<Node>, output: SurfaceOutput) -> ShaderGraph {
    ShaderGraph {
        surface: SurfaceGraph {
            nodes,
            output,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// A graph with a default surface and a displacement network holding `nodes`.
#[must_use]
pub fn displaced(nodes: Vec<Node>, position_offset: Option<Port>) -> ShaderGraph {
    ShaderGraph {
        surface: SurfaceGraph::default(),
        displacement: Some(DisplacementGraph {
            nodes,
            position_offset,
            normal_override: None,
            world_position_offset: None,
        }),
        ..Default::default()
    }
}
