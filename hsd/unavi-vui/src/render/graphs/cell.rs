//! What every cell graph shares: the look inputs, the three levels and how a
//! cell is drawn once its level is known.

use wired_guest::{
    math::{
        Color,
        Vec2,
    },
    style,
};

use super::{
    Net,
    binary,
    cf,
    with_alpha,
};
use crate::wired::shading::graph::{
    ExtractOp,
    GraphValue,
    LerpOp,
    Node,
    Port,
    SmoothstepOp,
};

/// How far the face is attended: `0` in the cell colour, `1` in the attended
/// colour.
pub const CELL_ATTEND: u16 = 0;
/// Where the pointer is, in cells from the face's top left.
pub const CELL_POINTER: u16 = 1;
/// How far the cell shape has flattened toward its average coverage, `0` for
/// squares and `1` for a grey picture.
pub const CELL_FLAT: u16 = 2;
/// The first input a graph declares for itself.
pub const CELL_OWN: u16 = 3;

/// A pointer too far from every cell to wake one.
const NO_POINTER: f32 = -1.0e4;
const NO_POINTER_AT: Vec2 = Vec2::new(NO_POINTER, NO_POINTER);
/// Weights of a cell's column and row in its breath's phase, so neighbours do
/// not breathe together.
const BREATH_PHASE: Vec2 = Vec2::new(1.7, 2.3);

/// What a face is told about its surroundings, as the shared inputs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Look {
    pub attend:  f32,
    /// In cells from the face's top left, or `None` for a pointer elsewhere.
    pub pointer: Option<Vec2>,
    pub flat:    f32,
}

impl Look {
    /// The override list, by input index.
    #[must_use]
    pub fn overrides(&self) -> Vec<(u16, GraphValue)> {
        vec![
            (CELL_ATTEND, GraphValue::Float(self.attend)),
            (
                CELL_POINTER,
                GraphValue::Vec2(self.pointer.unwrap_or(NO_POINTER_AT)),
            ),
            (CELL_FLAT, GraphValue::Float(self.flat)),
        ]
    }
}

/// The shared inputs' defaults, for a graph to start its list with.
pub fn look_defaults() -> Vec<GraphValue> {
    vec![
        GraphValue::Float(0.0),
        GraphValue::Vec2(NO_POINTER_AT),
        GraphValue::Float(0.0),
    ]
}

const fn colour(value: Color) -> Port {
    Port::Const(GraphValue::Color(value))
}

impl Net {
    /// The level of a cell that is off, or ember where `ember` is `1`, or lit
    /// where `lit` is `1`, as a fraction of the cell colour.
    pub(super) fn level(&mut self, ember: Port, lit: Port) -> Port {
        let to_ember = self.push(Node::Mul(binary(
            ember,
            cf(style::CELLS_LEVEL_EMBER - style::CELLS_LEVEL_OFF),
        )));
        let to_lit = self.push(Node::Mul(binary(
            lit,
            cf(style::CELLS_LEVEL_LIT - style::CELLS_LEVEL_EMBER),
        )));
        let ember_level = self.push(Node::Add(binary(cf(style::CELLS_LEVEL_OFF), to_ember)));
        self.push(Node::Add(binary(ember_level, to_lit)))
    }

    /// `level` with an off cell's breath added. `at` is the cell as a vec2
    /// and `off` is `1` for a cell that is off.
    pub(super) fn breathe(&mut self, level: Port, at: Port, off: Port) -> Port {
        let time = self.push(Node::Time);
        let turn = self.push(Node::Mul(binary(time, cf(style::CELLS_BREATH_RATE))));
        let spread = self.push(Node::Dot(binary(
            at,
            Port::Const(GraphValue::Vec2(BREATH_PHASE)),
        )));
        let phase = self.push(Node::Add(binary(turn, spread)));
        let wave = self.push(Node::Sin(phase));
        let swing = self.push(Node::Mul(binary(wave, cf(style::CELLS_BREATH))));
        let swing = self.push(Node::Mul(binary(swing, off)));
        self.push(Node::Add(binary(level, swing)))
    }

    /// `level` raised toward the pointer: the full boost under it, none at
    /// the wake radius. `centre` is the cell's centre in cells.
    pub(super) fn wake(&mut self, level: Port, centre: Port) -> Port {
        let apart = self.push(Node::Distance(binary(centre, Port::Input(CELL_POINTER))));
        let far = self.push(Node::Smoothstep(SmoothstepOp {
            low:  cf(0.0),
            high: cf(style::CELLS_WAKE_RADIUS as f32),
            x:    apart,
        }));
        let near = self.push(Node::OneMinus(far));
        let boost = self.push(Node::Mul(binary(near, cf(style::CELLS_WAKE_BOOST))));
        self.push(Node::Add(binary(level, boost)))
    }

    /// `1` inside the cell's square, `0` outside, eased over the softness
    /// token. `within` is where in the cell the fragment is, `0..1` each way.
    pub(super) fn square(&mut self, within: Port) -> Port {
        let from_middle = self.push(Node::Sub(binary(
            within,
            Port::Const(GraphValue::Vec2(Vec2::new(0.5, 0.5))),
        )));
        let across = self.push(Node::Extract(ExtractOp {
            v:       from_middle,
            channel: 0,
        }));
        let down = self.push(Node::Extract(ExtractOp {
            v:       from_middle,
            channel: 1,
        }));
        let across = self.push(Node::Abs(across));
        let down = self.push(Node::Abs(down));
        let reach = self.push(Node::Max(binary(across, down)));
        let half = style::CELLS_FILL / 2.0;
        // Rising edges only: a smoothstep with its edges the wrong way round
        // is undefined, so the shape is built rising and then inverted.
        let past = self.push(Node::Smoothstep(SmoothstepOp {
            low:  cf(half - style::CELLS_SOFT / 2.0),
            high: cf(half + style::CELLS_SOFT / 2.0),
            x:    reach,
        }));
        self.push(Node::OneMinus(past))
    }

    /// The finished colour: the cell colour, or the attended one, with the
    /// level in its alpha. The shape is flattened to its average coverage as
    /// [`CELL_FLAT`] rises.
    pub(super) fn paint(&mut self, level: Port, shape: Port) -> Port {
        let level = self.push(Node::Saturate(level));
        let squares = self.push(Node::Mul(binary(level, shape)));
        let average = self.push(Node::Mul(binary(
            level,
            cf(style::CELLS_FILL * style::CELLS_FILL),
        )));
        let alpha = self.push(Node::Lerp(LerpOp {
            a: squares,
            b: average,
            t: Port::Input(CELL_FLAT),
        }));
        let rgb = self.push(Node::Lerp(LerpOp {
            a: colour(style::CELLS_COLOUR),
            b: colour(style::CELLS_ATTENDED),
            t: Port::Input(CELL_ATTEND),
        }));
        with_alpha(self, rgb, alpha)
    }
}
