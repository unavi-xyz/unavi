//! The ground's shading: its colour, fading to nothing toward its edge so the
//! sky shows through where the slab ends.

use wired_guest::math::Color;

use crate::wired::shading::graph::{
    BinaryOp,
    BlendMode,
    CullMode,
    GraphValue,
    LitOutput,
    Node,
    Port,
    ShaderGraph,
    SmoothstepOp,
    SurfaceGraph,
    SurfaceOutput,
};

/// Metres from the slab's centre where the fade begins and where it is whole.
const FADE_START: f32 = 8.0;
const FADE_END: f32 = 14.0;

/// Public input 0 of [`graph`].
const COLOR: u16 = 0;

/// `color` lit as a matte surface, opaque inside [`FADE_START`] of the prim's
/// origin and clear past [`FADE_END`].
pub fn graph(color: Color, metallic: f32, roughness: f32) -> ShaderGraph {
    let nodes = vec![
        Node::WorldPosition,
        Node::ObjectPosition,
        Node::Distance(BinaryOp {
            a: Port::Node(0),
            b: Port::Node(1),
        }),
        Node::Smoothstep(SmoothstepOp {
            low: Port::Const(GraphValue::Float(FADE_START)),
            high: Port::Const(GraphValue::Float(FADE_END)),
            x: Port::Node(2),
        }),
        Node::OneMinus(Port::Node(3)),
    ];

    ShaderGraph {
        public_inputs: vec![GraphValue::Color(color)],
        surface: SurfaceGraph {
            nodes,
            output: SurfaceOutput::Lit(LitOutput {
                base_color: Some(Port::Input(COLOR)),
                emissive: None,
                metallic: Some(Port::Const(GraphValue::Float(metallic))),
                roughness: Some(Port::Const(GraphValue::Float(roughness))),
                normal: None,
                alpha: Some(Port::Node(4)),
                alpha_clip_threshold: None,
                specular_transmission: None,
                diffuse_transmission: None,
                thickness: None,
                ior: None,
            }),
            blend: BlendMode::Blend,
            cull: CullMode::Back,
            cast_shadows: false,
        },
        displacement: None,
    }
}
