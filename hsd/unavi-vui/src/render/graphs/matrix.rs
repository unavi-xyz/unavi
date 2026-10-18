//! The shading a glyph's dots are drawn with.
//!
//! A glyph is data, not a texture: its bits ride the graph's public inputs,
//! 24 to a float, and the graph reads them back out per fragment. That keeps a
//! face to a handful of override writes and lets one compiled program serve
//! every glyph there is.

use wired_guest::math::{
    Color,
    Vec2,
    Vec3,
};

use super::{
    Net,
    binary,
    cf,
    with_alpha,
};
use crate::{
    glyph::{
        CLIP_MARGIN,
        LANE_BITS,
        LANE_TRIPLES,
        Lanes,
    },
    wired::{
        scene::document::{
            Document,
            Layer,
        },
        shading::graph::{
            BlendMode,
            Combine2Op,
            CullMode,
            ExtractOp,
            GraphValue,
            LerpOp,
            Node,
            Port,
            SelectOp,
            ShaderGraph,
            SmoothstepOp,
            StepOp,
            SurfaceGraph,
            SurfaceOutput,
            UnlitOutput,
            set_graph,
        },
    },
};

/// The colour of a lit dot. Its alpha is not read.
pub const MATRIX_LIT: u16 = 0;
/// Cells along a side of the grid.
pub const MATRIX_SIZE: u16 = 1;
/// How far a flip has run, `0` not yet started and `1` over. Holds at `1`
/// when nothing is flipping, so a settled face shows its current bits alone.
pub const MATRIX_FLIP: u16 = 2;
/// How far the mote has come toward the attention it is under, which lifts
/// the whole face.
pub const MATRIX_HEAT: u16 = 3;
/// How far the mote has arrived, `0` not yet and `1` whole. Scales the whole
/// face, so the dots come up with the shell they sit on.
pub const MATRIX_BLOOM: u16 = 4;
/// The first of the vec3 inputs carrying the bits being shown, one per lane
/// triple.
pub const MATRIX_CURRENT: u16 = 5;
/// The first of the vec3 inputs carrying the bits a flip leaves.
pub const MATRIX_PREVIOUS: u16 = MATRIX_CURRENT + LANE_TRIPLES as u16;

/// How bright an unlit dot is held, against a lit one. Low enough that the
/// lit ones read as the picture and the unlit as the grid it is drawn on.
const UNLIT: f32 = 0.06;
/// Columns the wipe front takes to cross one column, so neighbours are
/// mid-flip together rather than turning one at a time.
const WIPE_WIDTH: f32 = 4.0;
/// How thin a dot gets at the edge of the flip. A dot squashed to nothing
/// would divide by zero, and a hairline reads as the dot turning edge on.
const SQUASH_FLOOR: f32 = 0.06;
/// Where a dot is solid and where it has faded out, as a fraction of a cell
/// from its centre.
const DOT_SOLID: f32 = 0.36;
const DOT_EDGE: f32 = 0.47;
/// Brightness of a face at rest and fully attended.
const GLOW_REST: f32 = 1.25;
const GLOW_ATTENDED: f32 = 2.4;

/// Where a fragment sits in the grid.
#[derive(Clone, Copy)]
struct Cell {
    /// Whole-number column and row.
    column: Port,
    row:    Port,
    /// The cell as a vec2 of those two.
    at:     Port,
    /// Where in the cell the fragment is, `0..1` each way.
    within: Port,
}

/// How far a column is through its flip.
#[derive(Clone, Copy)]
struct Turn {
    /// `1` while the column still shows the old bits, `0` once it shows the
    /// new.
    old:   Port,
    /// How wide its dots are right now, `1` at rest.
    width: Port,
}

impl Net {
    /// Which of a run of whole numbers `index` is, as a one-hot vector: `1` in
    /// the component whose number matches and `0` in the rest.
    ///
    /// Exact for a whole `index`, which is the point: a float cannot be used
    /// as an array index here, so a lookup is a dot product against one of
    /// these. `positions` holds the numbers the components stand for.
    fn one_hot(&mut self, index: Port, positions: GraphValue) -> Port {
        let away = self.push(Node::Sub(binary(index, Port::Const(positions))));
        let away = self.push(Node::Abs(away));
        let near = self.push(Node::OneMinus(away));
        self.push(Node::Saturate(near))
    }

    /// `table[index]`, for the three entries a `positions` vec3 stands for.
    fn pick3(&mut self, index: Port, first: f32, table: Vec3) -> Port {
        let positions = GraphValue::Vec3(Vec3::new(first, first + 1.0, first + 2.0));
        let hot = self.one_hot(index, positions);
        self.push(Node::Dot(binary(hot, Port::Const(GraphValue::Vec3(table)))))
    }

    /// The cell under the fragment.
    fn locate(&mut self, size: Port) -> Cell {
        let uv = self.push(Node::Uv);
        let scaled = self.push(Node::Mul(binary(uv, size)));
        let at = self.push(Node::Floor(scaled));
        let within = self.push(Node::Sub(binary(scaled, at)));
        let column = self.push(Node::Extract(ExtractOp {
            v:       at,
            channel: 0,
        }));
        let row = self.push(Node::Extract(ExtractOp {
            v:       at,
            channel: 1,
        }));
        Cell {
            column,
            row,
            at,
            within,
        }
    }

    /// `1` for a cell inside the clip circle, in the same numbers
    /// `glyph::inside` uses on the host.
    fn clip(&mut self, size: Port, cell: Cell) -> Port {
        let span = self.push(Node::Sub(binary(size, cf(1.0))));
        let half = self.push(Node::Mul(binary(span, cf(0.5))));
        let middle = self.push(Node::Combine2(Combine2Op { x: half, y: half }));
        let from_middle = self.push(Node::Sub(binary(cell.at, middle)));
        let apart = self.push(Node::Length(from_middle));
        let reach = self.push(Node::Add(binary(half, cf(CLIP_MARGIN))));
        self.push(Node::Step(StepOp {
            edge: apart,
            x:    reach,
        }))
    }

    /// Where `column` is in its flip.
    ///
    /// A wipe front crosses the grid left to right, taking `WIPE_WIDTH`
    /// columns to turn each one over. A column shows the old bits until it is
    /// half turned and the new after, and is narrowest at the halfway point.
    fn turn(&mut self, size: Port, column: Port) -> Turn {
        let travel = self.push(Node::Add(binary(size, cf(WIPE_WIDTH))));
        let front = self.push(Node::Mul(binary(Port::Input(MATRIX_FLIP), travel)));
        let past = self.push(Node::Sub(binary(front, column)));
        let turned = self.push(Node::Mul(binary(past, cf(1.0 / WIPE_WIDTH))));
        let turned = self.push(Node::Saturate(turned));
        let halfway = self.push(Node::Step(StepOp {
            edge: cf(0.5),
            x:    turned,
        }));
        let old = self.push(Node::OneMinus(halfway));
        let doubled = self.push(Node::Mul(binary(turned, cf(2.0))));
        let away = self.push(Node::OneMinus(doubled));
        let squash = self.push(Node::Abs(away));
        let width = self.push(Node::Max(binary(squash, cf(SQUASH_FLOOR))));
        Turn { old, width }
    }

    /// A soft disc about the cell's centre, squashed horizontally by `width`,
    /// solid inside `solid` of a cell and faded out by `edge`.
    fn disc(&mut self, cell: Cell, width: Port, solid: f32, edge: f32) -> Port {
        let local = self.push(Node::Sub(binary(
            cell.within,
            Port::Const(GraphValue::Vec2(Vec2::new(0.5, 0.5))),
        )));
        let across = self.push(Node::Extract(ExtractOp {
            v:       local,
            channel: 0,
        }));
        let down = self.push(Node::Extract(ExtractOp {
            v:       local,
            channel: 1,
        }));
        let squashed = self.push(Node::Div(binary(across, width)));
        let offset = self.push(Node::Combine2(Combine2Op {
            x: squashed,
            y: down,
        }));
        let radius = self.push(Node::Length(offset));
        // Rising edges only: a smoothstep with its edges the wrong way round
        // is undefined, so the fade is built rising and then inverted.
        let fade = self.push(Node::Smoothstep(SmoothstepOp {
            low:  cf(solid),
            high: cf(edge),
            x:    radius,
        }));
        self.push(Node::OneMinus(fade))
    }

    /// The value of the lane holding `cell`'s bit, from whichever glyph the
    /// column is showing, and the bit's place within that lane.
    ///
    /// The cell number is split into the lane and the place within it. A half
    /// is added before dividing so a cell on a lane boundary lands inside its
    /// lane whatever the divide rounds to.
    fn lane_value(&mut self, size: Port, cell: Cell, old: Port) -> (Port, Port) {
        let by_row = self.push(Node::Mul(binary(cell.row, size)));
        let index = self.push(Node::Add(binary(by_row, cell.column)));
        let centred = self.push(Node::Add(binary(index, cf(0.5))));
        let over = self.push(Node::Mul(binary(centred, cf(1.0 / LANE_BITS as f32))));
        let lane = self.push(Node::Floor(over));
        let before = self.push(Node::Mul(binary(lane, cf(LANE_BITS as f32))));
        let place = self.push(Node::Sub(binary(index, before)));

        let held = (0..LANE_TRIPLES as u16)
            .map(|triple| {
                let shown = self.push(Node::Select(SelectOp {
                    cond: old,
                    a:    Port::Input(MATRIX_PREVIOUS + triple),
                    b:    Port::Input(MATRIX_CURRENT + triple),
                }));
                let first = f32::from(triple * 3);
                let positions = GraphValue::Vec3(Vec3::new(first, first + 1.0, first + 2.0));
                let hot = self.one_hot(lane, positions);
                self.push(Node::Dot(binary(shown, hot)))
            })
            .collect::<Vec<_>>();
        let value = held
            .into_iter()
            .reduce(|sum, piece| self.push(Node::Add(binary(sum, piece))))
            .unwrap_or(cf(0.0));
        (value, place)
    }

    /// `2^-place`, picked rather than computed. The place splits into groups
    /// of four and a remainder, each a short table.
    fn unshift(&mut self, place: Port) -> Port {
        let quarter = self.push(Node::Mul(binary(place, cf(0.25))));
        let group = self.push(Node::Floor(quarter));
        let group_start = self.push(Node::Mul(binary(group, cf(4.0))));
        let rest = self.push(Node::Sub(binary(place, group_start)));
        let rest_hot = self.one_hot(
            rest,
            GraphValue::Color(Color {
                r: 0.0,
                g: 1.0,
                b: 2.0,
                a: 3.0,
            }),
        );
        let fine = self.push(Node::Dot(binary(
            rest_hot,
            Port::Const(GraphValue::Color(Color {
                r: 1.0,
                g: 0.5,
                b: 0.25,
                a: 0.125,
            })),
        )));
        let near = self.pick3(group, 0.0, Vec3::new(1.0, 1.0 / 16.0, 1.0 / 256.0));
        let far = self.pick3(
            group,
            3.0,
            Vec3::new(1.0 / 4096.0, 1.0 / 65_536.0, 1.0 / 1_048_576.0),
        );
        let coarse = self.push(Node::Add(binary(near, far)));
        self.push(Node::Mul(binary(fine, coarse)))
    }

    /// `1` if the bit of `value` at the place `scale` unshifts is set: shift
    /// it down to the ones place, then take the parity.
    fn bit(&mut self, value: Port, scale: Port) -> Port {
        let shifted = self.push(Node::Mul(binary(value, scale)));
        let whole = self.push(Node::Floor(shifted));
        let half_of = self.push(Node::Mul(binary(whole, cf(0.5))));
        let parity = self.push(Node::Fract(half_of));
        self.push(Node::Step(StepOp {
            edge: cf(0.5),
            x:    parity,
        }))
    }
}

/// One dot of a glyph, shaded.
///
/// Each fragment finds its cell from the face's UV, decides from the cell's
/// column how far through a flip it is, reads its bit out of the current or
/// the previous glyph accordingly, and draws a soft disc scaled to match.
///
/// The bit lookup is arithmetic on exact whole numbers throughout: lanes
/// are picked by dot product against a one-hot vector, and the bit within a
/// lane by scaling by a power of two that is itself picked the same way, so no
/// step depends on a transcendental function being exact.
///
/// Additive with alpha 0, like the cast ring: the face adds light to the glass
/// behind it rather than covering it.
pub fn matrix() -> ShaderGraph {
    let mut net = Net::default();
    let size = Port::Input(MATRIX_SIZE);

    let cell = net.locate(size);
    let inside = net.clip(size, cell);
    let turn = net.turn(size, cell.column);
    let dot = net.disc(cell, turn.width, DOT_SOLID, DOT_EDGE);
    let (value, place) = net.lane_value(size, cell, turn.old);
    let scale = net.unshift(place);
    let lit = net.bit(value, scale);

    let level = net.push(Node::Lerp(LerpOp {
        a: cf(UNLIT),
        b: cf(1.0),
        t: lit,
    }));
    let glow = net.push(Node::Lerp(LerpOp {
        a: cf(GLOW_REST),
        b: cf(GLOW_ATTENDED),
        t: Port::Input(MATRIX_HEAT),
    }));
    let bright = net.push(Node::Mul(binary(level, glow)));
    let bright = net.push(Node::Mul(binary(bright, Port::Input(MATRIX_BLOOM))));
    let shaped = net.push(Node::Mul(binary(dot, inside)));
    let drawn = net.push(Node::Mul(binary(shaped, bright)));
    let rgb = net.push(Node::Mul(binary(Port::Input(MATRIX_LIT), drawn)));
    // Alpha 0 for the same reason the ring has it: additive blending keeps
    // `dst * (1 - src.a)`, so any alpha would darken the glass under a dot's
    // soft edge.
    let color = with_alpha(&mut net, rgb, cf(0.0));

    let mut public_inputs = vec![
        GraphValue::Color(Color {
            r: 0.93,
            g: 0.95,
            b: 0.97,
            a: 1.0,
        }),
        GraphValue::Float(9.0),
        GraphValue::Float(1.0),
        GraphValue::Float(0.0),
        GraphValue::Float(1.0),
    ];
    public_inputs.extend((0..LANE_TRIPLES * 2).map(|_| GraphValue::Vec3(Vec3::ZERO)));

    ShaderGraph {
        public_inputs,
        surface: SurfaceGraph {
            nodes:        net.nodes,
            output:       SurfaceOutput::Unlit(UnlitOutput {
                color,
                alpha_clip_threshold: None,
            }),
            blend:        BlendMode::Add,
            cull:         CullMode::None,
            cast_shadows: false,
        },
        displacement: None,
    }
}

/// Submits [`matrix`] to `prim`.
pub fn set_matrix(doc: &Document, prim: (u64, u64)) -> anyhow::Result<()> {
    set_graph(doc, prim, Layer::Local, Some(&matrix()))?;
    Ok(())
}

/// Everything [`matrix`] is handed for one face. Comparable, so a face that
/// has not changed since it was last written is not written again.
#[derive(Debug, Clone, PartialEq)]
pub struct Inputs {
    pub lit:      Color,
    pub size:     usize,
    pub flip:     f32,
    pub heat:     f32,
    pub bloom:    f32,
    pub current:  Lanes,
    pub previous: Lanes,
}

impl Inputs {
    /// The override list for `set_overrides`, by input index.
    #[must_use]
    pub fn overrides(&self) -> Vec<(u16, GraphValue)> {
        let mut values = vec![
            (MATRIX_LIT, GraphValue::Color(self.lit)),
            (MATRIX_SIZE, GraphValue::Float(self.size as f32)),
            (MATRIX_FLIP, GraphValue::Float(self.flip)),
            (MATRIX_HEAT, GraphValue::Float(self.heat)),
            (MATRIX_BLOOM, GraphValue::Float(self.bloom)),
        ];
        for (base, lanes) in [
            (MATRIX_CURRENT, &self.current),
            (MATRIX_PREVIOUS, &self.previous),
        ] {
            values.extend(lanes.iter().zip(base..).map(|(lane, input)| {
                (
                    input,
                    GraphValue::Vec3(Vec3::new(lane[0], lane[1], lane[2])),
                )
            }));
        }
        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glyph::Glyph;

    /// A value the graph computes: its components, and how many it has,
    /// which stands in for its kind (1 float, 2 vec2, 3 vec3, 4 colour).
    #[derive(Clone, Copy, Debug)]
    struct Val {
        n: usize,
        c: [f32; 4],
    }

    fn of(value: &GraphValue) -> Val {
        match value {
            GraphValue::Float(x) => Val {
                n: 1,
                c: [*x, 0.0, 0.0, 0.0],
            },
            GraphValue::Vec2(v) => Val {
                n: 2,
                c: [v.x, v.y, 0.0, 0.0],
            },
            GraphValue::Vec3(v) => Val {
                n: 3,
                c: [v.x, v.y, v.z, 0.0],
            },
            GraphValue::Color(v) => Val {
                n: 4,
                c: [v.r, v.g, v.b, v.a],
            },
        }
    }

    fn map(a: Val, f: impl Fn(f32) -> f32) -> Val {
        Val {
            n: a.n,
            c: a.c.map(&f),
        }
    }

    /// `+ - * /` pair two of one kind or broadcast a float; anything else is
    /// a graph the host would refuse.
    fn arithmetic(left: Val, right: Val, op: impl Fn(f32, f32) -> f32) -> Val {
        assert!(
            left.n == right.n || left.n == 1 || right.n == 1,
            "{} with {}",
            left.n,
            right.n
        );
        let n = left.n.max(right.n);
        let mut c = [0.0; 4];
        for (component, out) in c.iter_mut().enumerate().take(n) {
            *out = op(
                left.c[component.min(left.n - 1)],
                right.c[component.min(right.n - 1)],
            );
        }
        Val { n, c }
    }

    /// Operands that must be one kind exactly.
    fn matching(a: Val, b: Val, f: impl Fn(f32, f32) -> f32) -> Val {
        assert_eq!(a.n, b.n, "operands of different kinds");
        arithmetic(a, b, f)
    }

    fn gather(vals: &[Val], port: Port, inputs: &[GraphValue]) -> Val {
        match port {
            Port::Const(value) => of(&value),
            Port::Input(i) => of(&inputs[usize::from(i)]),
            Port::Node(i) => vals[usize::from(i)],
        }
    }

    /// Runs the fragment network at `uv` the way the host would, for the
    /// subset of nodes the matrix uses. Checks the host's kind rules on the
    /// way, which a guest otherwise only learns of from a refused graph.
    fn shade(graph: &ShaderGraph, inputs: &[GraphValue], uv: [f32; 2]) -> Val {
        let mut vals: Vec<Val> = Vec::new();
        for node in &graph.surface.nodes {
            let at = |port: Port| gather(&vals, port, inputs);
            let out = match node {
                Node::Uv => Val {
                    n: 2,
                    c: [uv[0], uv[1], 0.0, 0.0],
                },
                Node::Add(op) => arithmetic(at(op.a), at(op.b), |x, y| x + y),
                Node::Sub(op) => arithmetic(at(op.a), at(op.b), |x, y| x - y),
                Node::Mul(op) => arithmetic(at(op.a), at(op.b), |x, y| x * y),
                Node::Div(op) => arithmetic(at(op.a), at(op.b), |x, y| x / y),
                Node::Max(op) => matching(at(op.a), at(op.b), f32::max),
                Node::Floor(p) => map(at(*p), f32::floor),
                Node::Fract(p) => map(at(*p), |x| x - x.floor()),
                Node::Abs(p) => map(at(*p), f32::abs),
                Node::OneMinus(p) => map(at(*p), |x| 1.0 - x),
                Node::Saturate(p) => map(at(*p), |x| x.clamp(0.0, 1.0)),
                Node::Length(p) => {
                    let v = at(*p);
                    assert!(v.n > 1, "length of a scalar");
                    Val {
                        n: 1,
                        c: [v.c.iter().map(|x| x * x).sum::<f32>().sqrt(), 0.0, 0.0, 0.0],
                    }
                }
                Node::Dot(op) => {
                    let v = matching(at(op.a), at(op.b), |x, y| x * y);
                    Val {
                        n: 1,
                        c: [v.c.iter().take(v.n).sum(), 0.0, 0.0, 0.0],
                    }
                }
                Node::Step(op) => matching(at(op.edge), at(op.x), |e, x| f32::from(x >= e)),
                Node::Smoothstep(op) => {
                    let (low, high, x) = (at(op.low), at(op.high), at(op.x));
                    let first = matching(low, high, |l, h| h - l);
                    let t = matching(first, matching(x, low, |x, l| x - l), |w, d| {
                        (d / w).clamp(0.0, 1.0)
                    });
                    map(t, |t| t * t * 2.0f32.mul_add(-t, 3.0))
                }
                Node::Lerp(op) => {
                    let (a, b, t) = (at(op.a), at(op.b), at(op.t));
                    assert_eq!(t.n, 1, "lerp factor must be a float");
                    assert_eq!(a.n, b.n);
                    arithmetic(a, arithmetic(b, a, |b, a| b - a), |a, d| {
                        d.mul_add(t.c[0], a)
                    })
                }
                Node::Select(op) => {
                    let (cond, a, b) = (at(op.cond), at(op.a), at(op.b));
                    assert_eq!(cond.n, 1, "select condition must be a float");
                    assert_eq!(a.n, b.n);
                    if cond.c[0] > 0.5 { a } else { b }
                }
                Node::Extract(op) => {
                    let v = at(op.v);
                    assert!(v.n > 1 && usize::from(op.channel) < v.n);
                    Val {
                        n: 1,
                        c: [v.c[usize::from(op.channel)], 0.0, 0.0, 0.0],
                    }
                }
                Node::Combine2(op) => {
                    let (x, y) = (at(op.x), at(op.y));
                    assert!(x.n == 1 && y.n == 1);
                    Val {
                        n: 2,
                        c: [x.c[0], y.c[0], 0.0, 0.0],
                    }
                }
                Node::Combine4(op) => {
                    let parts = [at(op.x), at(op.y), at(op.z), at(op.w)];
                    assert!(parts.iter().all(|part| part.n == 1));
                    Val {
                        n: 4,
                        c: parts.map(|part| part.c[0]),
                    }
                }
                other => panic!("the matrix does not use {other:?}"),
            };
            vals.push(out);
        }
        let SurfaceOutput::Unlit(unlit) = &graph.surface.output else {
            panic!("the matrix is unlit");
        };
        gather(&vals, unlit.color, inputs)
    }

    fn glyph(size: u8, lit: &[(usize, usize)]) -> Glyph {
        let edge = usize::from(size);
        let mut bits = vec![0_u8; (edge * edge).div_ceil(8)];
        for (column, row) in lit {
            let cell = row * edge + column;
            bits[cell / 8] |= 1 << (cell % 8);
        }
        Glyph::new(size, &bits).expect("a well-formed glyph")
    }

    fn inputs(current: &Glyph, previous: &Glyph, flip: f32) -> Vec<GraphValue> {
        let mut values = matrix().public_inputs;
        let face = Inputs {
            lit: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            size: current.size(),
            flip,
            heat: 0.0,
            bloom: 1.0,
            current: *current.lanes(),
            previous: *previous.lanes(),
        };
        for (input, value) in face.overrides() {
            values[usize::from(input)] = value;
        }
        values
    }

    /// The light a cell gives at `across` cells right of its centre.
    fn light(values: &[GraphValue], size: usize, column: usize, row: usize, across: f32) -> f32 {
        let at = [
            (column as f32 + 0.5 + across) / size as f32,
            (row as f32 + 0.5) / size as f32,
        ];
        let out = shade(&matrix(), values, at);
        assert!(out.c[3].abs() < f32::EPSILON, "additive, so alpha 0");
        out.c[0]
    }

    /// A glyph with a mix of lit and unlit cells in every lane, so a wrong
    /// lane or bit shows as a wrong cell.
    fn pattern(size: usize) -> Vec<(usize, usize)> {
        (0..size * size)
            .filter(|cell| (cell * 7 + cell / 3) % 5 < 2)
            .map(|cell| (cell % size, cell / size))
            .collect()
    }

    #[test]
    fn every_cell_reads_back_as_the_glyph_says_for_both_sizes() {
        for size in [9_u8, 13] {
            let side = usize::from(size);
            let lit = pattern(side);
            let face = glyph(size, &lit);
            let values = inputs(&face, &face, 1.0);
            for row in 0..side {
                for column in 0..side {
                    let seen = light(&values, side, column, row, 0.0);
                    let expect = if !crate::glyph::inside(side, column, row) {
                        0.0
                    } else if lit.contains(&(column, row)) {
                        GLOW_REST
                    } else {
                        UNLIT * GLOW_REST
                    };
                    assert!(
                        (seen - expect).abs() < 1.0e-4,
                        "{size}: cell {column},{row} drew {seen}, wanted {expect}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_lit_dot_stands_well_clear_of_an_unlit_one_and_stays_in_its_cell() {
        const {
            assert!(
                GLOW_REST / (UNLIT * GLOW_REST) >= 10.0,
                "lit against unlit is the whole picture"
            );
            assert!(
                DOT_SOLID >= 0.35,
                "a dot with a small core reads as a speck at a distance"
            );
            assert!(
                DOT_EDGE < 0.5,
                "a dot that fades out past its cell runs into its neighbours"
            );
        }
    }

    #[test]
    fn a_dot_fades_out_before_its_cell_ends() {
        let face = glyph(9, &[(4, 4)]);
        let values = inputs(&face, &face, 1.0);
        assert!(light(&values, 9, 4, 4, 0.0) > 0.8);
        assert!(light(&values, 9, 4, 4, 0.49) < 1.0e-4);
    }

    #[test]
    fn a_flip_starts_on_the_old_bits_and_ends_on_the_new() {
        let (old, new) = (glyph(9, &[(4, 4), (3, 4)]), glyph(9, &[(4, 4), (5, 4)]));
        let start = inputs(&new, &old, 0.0);
        let end = inputs(&new, &old, 1.0);
        assert!(light(&start, 9, 3, 4, 0.0) > 0.8, "the old dot still there");
        assert!(light(&start, 9, 5, 4, 0.0) < 0.2, "the new one not yet");
        assert!(light(&end, 9, 3, 4, 0.0) < 0.2);
        assert!(light(&end, 9, 5, 4, 0.0) > 0.8);
    }

    #[test]
    fn a_flip_wipes_across_the_columns_from_the_left() {
        // Lit in every column before, and in none after.
        let before = (0..9).map(|column| (column, 4)).collect::<Vec<_>>();
        let (old, new) = (glyph(9, &before), glyph(9, &[]));
        let values = inputs(&new, &old, 0.5);

        // The front is at 0.5 * (9 + 4) = 6.5 columns, and a column turns over
        // once the front is two columns past it, so columns 0 to 4 are new
        // (dark) and 5 to 8 are still old (lit).
        let lit = (0..9)
            .map(|column| light(&values, 9, column, 4, 0.0) > 0.5)
            .collect::<Vec<_>>();
        let expected = (0..9).map(|column| column >= 5).collect::<Vec<_>>();
        assert_eq!(lit, expected);
    }

    #[test]
    fn a_dot_narrows_through_its_flip_and_widens_again() {
        let face = glyph(9, &[(2, 4)]);
        // Column 2 is half turned when the front is two columns past it:
        // flip * (9 + 4) = 2 + 2.
        let values = inputs(&face, &face, 4.0 / 13.0);
        assert!(
            light(&values, 9, 2, 4, 0.0) > 0.5,
            "still a dot at its centre"
        );
        assert!(
            light(&values, 9, 2, 4, 0.2) < 1.0e-4,
            "but squashed past a fifth of a cell off it"
        );
        let settled = inputs(&face, &face, 1.0);
        assert!(light(&settled, 9, 2, 4, 0.2) > 0.5, "full width again");
    }

    #[test]
    fn heat_lifts_a_lit_dot() {
        let face = glyph(9, &[(4, 4)]);
        let mut values = inputs(&face, &face, 1.0);
        let cool = light(&values, 9, 4, 4, 0.0);
        values[usize::from(MATRIX_HEAT)] = GraphValue::Float(1.0);
        assert!(light(&values, 9, 4, 4, 0.0) > cool);
    }

    #[test]
    fn a_face_comes_up_with_its_shell() {
        let face = glyph(9, &[(4, 4)]);
        let mut values = inputs(&face, &face, 1.0);
        let whole = light(&values, 9, 4, 4, 0.0);
        values[usize::from(MATRIX_BLOOM)] = GraphValue::Float(0.5);
        let half = light(&values, 9, 4, 4, 0.0);
        assert!(whole.mul_add(-0.5, half).abs() < 1.0e-5);
        // Unlit dots arrive with it too.
        assert!(light(&values, 9, 3, 4, 0.0) < UNLIT * GLOW_REST);
        values[usize::from(MATRIX_BLOOM)] = GraphValue::Float(0.0);
        assert!(light(&values, 9, 4, 4, 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn the_override_list_names_every_input_once() {
        let face = glyph(9, &[]);
        let list = Inputs {
            lit:      Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            size:     9,
            flip:     1.0,
            heat:     0.0,
            bloom:    1.0,
            current:  *face.lanes(),
            previous: *face.lanes(),
        }
        .overrides();
        let mut named = list.iter().map(|(input, _)| *input).collect::<Vec<_>>();
        named.sort_unstable();
        let all = (0..matrix().public_inputs.len() as u16).collect::<Vec<_>>();
        assert_eq!(named, all);
    }
}
