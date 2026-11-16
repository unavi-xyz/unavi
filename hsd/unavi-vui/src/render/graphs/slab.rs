//! The Frost slab: frosted glass over a rounded rectangle, with a bright edge
//! and a faint hairline inside it.
//!
//! Every length is a fraction of the slab's short side, so a slab keeps its
//! proportions at any size.

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
use crate::wired::{
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
        ShaderGraph,
        SmoothstepOp,
        StepOp,
        SurfaceGraph,
        SurfaceOutput,
        UnlitOutput,
        set_graph,
        set_overrides,
    },
};

/// How far a slab has unrolled from its top edge, 0 for not at all and 1 for
/// whole. Drawn as a front sweeping down it, so a slab opens and closes like a
/// blind rather than fading, which opaque glass cannot.
pub const SLAB_OPEN: u16 = 0;

/// How far the blur's diagonal taps sit from the centre, against the axis
/// ones, so the nine taps cover a disc.
const DIAGONAL: f32 = 0.75;

const fn colour(value: Color) -> Port {
    Port::Const(GraphValue::Color(value))
}

impl Net {
    /// The room behind a slab, blurred over nine taps, under the frost's
    /// white ground and its fill.
    fn frosted_room(&mut self) -> Port {
        let screen = self.push(Node::ScreenUv);
        let mut sum = self.push(Node::SceneColor(screen));
        let spread = style::FROST_BLUR_UV;
        let diagonal = spread * DIAGONAL;
        let taps = [
            (spread, 0.0),
            (-spread, 0.0),
            (0.0, spread),
            (0.0, -spread),
            (diagonal, diagonal),
            (-diagonal, diagonal),
            (diagonal, -diagonal),
            (-diagonal, -diagonal),
        ];
        for (dx, dy) in taps {
            let at = self.push(Node::Add(binary(
                screen,
                Port::Const(GraphValue::Vec2(Vec2::new(dx, dy))),
            )));
            let tap = self.push(Node::SceneColor(at));
            sum = self.push(Node::Add(binary(sum, tap)));
        }
        let blurred = self.push(Node::Mul(binary(sum, cf(1.0 / 9.0))));
        let ground = self.push(Node::Lerp(LerpOp {
            a: blurred,
            b: colour(style::FROST_GROUND),
            t: cf(style::FROST_GROUND_ALPHA),
        }));
        self.push(Node::Lerp(LerpOp {
            a: ground,
            b: colour(style::FROST_FILL),
            t: cf(style::FROST_FILL_ALPHA),
        }))
    }

    /// Metres from the fragment to each border of the slab: left and right,
    /// bottom and top.
    fn borders(&mut self, scale: Port) -> [Port; 4] {
        let uv = self.push(Node::Uv);
        let axis = |net: &mut Self, channel: u8| {
            let across = net.push(Node::Extract(ExtractOp { v: uv, channel }));
            let back = net.push(Node::OneMinus(across));
            let size = net.push(Node::Extract(ExtractOp { v: scale, channel }));
            let low = net.push(Node::Mul(binary(across, size)));
            let high = net.push(Node::Mul(binary(back, size)));
            (low, high)
        };
        let (left, right) = axis(self, 0);
        let (bottom, top) = axis(self, 1);
        [left, right, bottom, top]
    }

    /// Metres inside a rounded rectangle, negative outside, capped at the
    /// corner radius: the nearer border's distance, except within a corner's
    /// radius of two, where it is the radius less the distance from the
    /// corner circle's centre.
    fn depth(&mut self, [left, right, bottom, top]: [Port; 4], corner: Port) -> Port {
        let wide = self.push(Node::Min(binary(left, right)));
        let tall = self.push(Node::Min(binary(bottom, top)));
        let straight = self.push(Node::Min(binary(wide, tall)));
        let round = |net: &mut Self, depth: Port| {
            let short = net.push(Node::Sub(binary(corner, depth)));
            net.push(Node::Max(binary(short, cf(0.0))))
        };
        let round_wide = round(self, wide);
        let round_tall = round(self, tall);
        let corner_at = self.push(Node::Combine2(Combine2Op {
            x: round_wide,
            y: round_tall,
        }));
        let reach = self.push(Node::Length(corner_at));
        let rounded = self.push(Node::Sub(binary(corner, reach)));
        self.push(Node::Min(binary(straight, rounded)))
    }
}

/// A Frost slab. The room behind it is blurred and lifted by the frost's
/// white ground, a bright edge runs round its border and a hairline just
/// inside that.
///
/// The shape is an alpha clip on a flat quad rather than geometry. Like the
/// shell it is opaque so it reaches the phase `SceneColor` is filled in, and
/// unlit so the lit transmissive path cannot occlude it. It is double sided
/// so a slab reads from whichever side the viewer stands.
pub fn slab() -> ShaderGraph {
    let mut net = Net::default();
    let frosted = net.frosted_room();

    let scale = net.push(Node::ObjectScale);
    let wide = net.push(Node::Extract(ExtractOp {
        v:       scale,
        channel: 0,
    }));
    let tall = net.push(Node::Extract(ExtractOp {
        v:       scale,
        channel: 1,
    }));
    let short = net.push(Node::Min(binary(wide, tall)));
    let corner = net.push(Node::Mul(binary(short, cf(style::FROST_RADIUS))));
    let edge = net.push(Node::Mul(binary(short, cf(style::FROST_EDGE_WIDTH))));

    let borders = net.borders(scale);
    let depth = net.depth(borders, corner);
    let body = net.push(Node::Step(StepOp {
        edge: cf(0.0),
        x:    depth,
    }));

    // The edge is the first band of depth and the hairline the second; each
    // eases at both of its sides.
    let half_edge = net.push(Node::Mul(binary(edge, cf(0.5))));
    let inside_edge = net.push(Node::Smoothstep(SmoothstepOp {
        low:  half_edge,
        high: edge,
        x:    depth,
    }));
    let on_edge = net.push(Node::OneMinus(inside_edge));
    let edge_t = net.push(Node::Mul(binary(on_edge, cf(style::FROST_EDGE_ALPHA))));
    let past_edge = net.push(Node::Mul(binary(edge, cf(1.5))));
    let past_line = net.push(Node::Mul(binary(edge, cf(2.0))));
    let beyond_line = net.push(Node::Smoothstep(SmoothstepOp {
        low:  past_edge,
        high: past_line,
        x:    depth,
    }));
    let in_line = net.push(Node::OneMinus(beyond_line));
    let on_line = net.push(Node::Mul(binary(inside_edge, in_line)));
    let line_t = net.push(Node::Mul(binary(on_line, cf(style::FROST_HAIRLINE_ALPHA))));
    let lined = net.push(Node::Lerp(LerpOp {
        a: frosted,
        b: colour(style::FROST_HAIRLINE),
        t: line_t,
    }));
    let painted = net.push(Node::Lerp(LerpOp {
        a: lined,
        b: colour(style::FROST_EDGE),
        t: edge_t,
    }));

    let (painted, covered) = net.unroll(painted, body, (half_edge, edge));
    let color = with_alpha(&mut net, painted, covered);

    ShaderGraph {
        public_inputs: vec![GraphValue::Float(1.0)],
        surface:       SurfaceGraph {
            nodes:        net.nodes,
            // Clipping composes with the opaque blend, so the cut shape does
            // not leave the phase `SceneColor` is filled in.
            output:       SurfaceOutput::Unlit(UnlitOutput {
                color,
                alpha_clip_threshold: Some(cf(0.5)),
            }),
            blend:        BlendMode::Opaque,
            cull:         CullMode::None,
            cast_shadows: false,
        },
        displacement:  None,
    }
}

impl Net {
    /// Unrolls a slab drawn as `painted` over `covered` down to
    /// [`SLAB_OPEN`]: everything nearer the top edge than the front shows,
    /// and while it moves, a bright line `edge` metres wide runs along the
    /// front.
    fn unroll(&mut self, painted: Port, covered: Port, edge: (Port, Port)) -> (Port, Port) {
        // Metres below the top edge, measured as the shape is.
        let uv = self.push(Node::Uv);
        let down = self.push(Node::Extract(ExtractOp {
            v:       uv,
            channel: 1,
        }));
        let down = self.push(Node::OneMinus(down));
        let scale = self.push(Node::ObjectScale);
        let tall = self.push(Node::Extract(ExtractOp {
            v:       scale,
            channel: 1,
        }));
        let top = self.push(Node::Mul(binary(down, tall)));
        let front = self.push(Node::Mul(binary(Port::Input(SLAB_OPEN), tall)));
        let unrolled = self.push(Node::Step(StepOp {
            edge: top,
            x:    front,
        }));
        let behind = self.push(Node::Sub(binary(front, top)));
        let past_front = self.push(Node::Smoothstep(SmoothstepOp {
            low:  edge.0,
            high: edge.1,
            x:    behind,
        }));
        let near_front = self.push(Node::OneMinus(past_front));
        let opening = self.push(Node::OneMinus(Port::Input(SLAB_OPEN)));
        let opening = self.push(Node::Step(StepOp {
            edge: cf(1.0e-3),
            x:    opening,
        }));
        let sweep = self.push(Node::Mul(binary(near_front, opening)));
        let painted = self.push(Node::Lerp(LerpOp {
            a: painted,
            b: colour(style::FROST_EDGE),
            t: sweep,
        }));
        let covered = self.push(Node::Mul(binary(covered, unrolled)));
        (painted, covered)
    }
}

/// Submits [`slab`] to `prim`.
pub fn set_slab(doc: &Document, prim: (u64, u64)) -> anyhow::Result<()> {
    set_graph(doc, prim, Layer::Local, Some(&slab()))?;
    Ok(())
}

/// How far `prim`'s slab has unrolled; see [`SLAB_OPEN`].
pub fn set_slab_open(doc: &Document, prim: (u64, u64), open: f32) -> anyhow::Result<()> {
    set_overrides(
        doc,
        prim,
        Layer::Local,
        &[(SLAB_OPEN, GraphValue::Float(open.clamp(0.0, 1.0)))],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        super::interp::{
            self,
            Context,
            Shaded,
        },
        *,
    };

    /// A slab 0.6 m by 0.3 m, so 0.3 m is the short side, over a backdrop of
    /// one colour.
    const WIDE: f32 = 0.6;
    const TALL: f32 = 0.3;

    fn lift(from: f32, to: f32, by: f32) -> f32 {
        (to - from).mul_add(by, from)
    }

    fn shade_at(open: f32, scene: f32, uv: [f32; 2]) -> Shaded {
        let graph = slab();
        let mut values = graph.public_inputs.clone();
        values[usize::from(SLAB_OPEN)] = GraphValue::Float(open);
        let context = Context {
            scale: [WIDE, TALL, 1.0],
            scene: [scene, scene, scene, 1.0],
            ..Context::at(uv)
        };
        interp::shade(&graph, &values, &context)
    }

    /// The slab at a point `metres` in from the left edge, mid height.
    fn from_left(metres: f32, scene: f32) -> Shaded {
        shade_at(1.0, scene, [metres / WIDE, 0.5])
    }

    #[test]
    fn the_slab_is_opaque_unlit_clipped_and_double_sided() {
        let graph = slab();
        assert!(matches!(graph.surface.blend, BlendMode::Opaque));
        assert!(matches!(graph.surface.cull, CullMode::None));
        let SurfaceOutput::Unlit(unlit) = graph.surface.output else {
            panic!("the slab is unlit");
        };
        assert!(unlit.alpha_clip_threshold.is_some());
    }

    /// Frost is the room behind the slab, so it has to be sampled in the
    /// phase that fills it, and several times or it is not blurred at all.
    #[test]
    fn the_slab_blurs_the_room_over_nine_taps() {
        let taps = slab()
            .surface
            .nodes
            .iter()
            .filter(|node| matches!(node, Node::SceneColor(_)))
            .count();
        assert_eq!(taps, 9);
    }

    #[test]
    fn the_interior_is_the_room_under_the_frost_ground_and_fill() {
        for scene in [0.0, 1.0] {
            let seen = shade_at(1.0, scene, [0.5, 0.5]);
            assert!(!seen.discarded());
            let ground = lift(scene, style::FROST_GROUND.r, style::FROST_GROUND_ALPHA);
            let expected = lift(ground, style::FROST_FILL.r, style::FROST_FILL_ALPHA);
            assert!(
                (seen.color.c[0] - expected).abs() < 1.0e-5,
                "over {scene}: {} not {expected}",
                seen.color.c[0]
            );
        }
    }

    #[test]
    fn the_corners_are_round_at_the_token_radius() {
        let radius = style::FROST_RADIUS * TALL;
        let near = |x: f32, y: f32| shade_at(1.0, 0.0, [x / WIDE, y / TALL]).discarded();
        assert!(near(0.0005, 0.0005), "a hair from both borders is cut away");
        assert!(!near(radius, radius), "the circle's centre is inside");
        // On the arc, 45 degrees out from the centre.
        let diagonal = radius * (1.0 - std::f32::consts::FRAC_1_SQRT_2);
        assert!(near(diagonal * 0.5, diagonal * 0.5), "outside the arc");
        assert!(!near(diagonal * 1.5, diagonal * 1.5), "inside the arc");
        assert!(!near(0.0005, TALL / 2.0), "a straight edge is not rounded");
    }

    #[test]
    fn an_edge_of_the_token_width_runs_round_the_border_and_a_hairline_inside_it() {
        let edge = style::FROST_EDGE_WIDTH * TALL;
        let on_edge = from_left(edge * 0.25, 0.0).color.c[0];
        let on_line = from_left(edge * 1.25, 0.0).color.c[0];
        let interior = from_left(edge * 4.0, 0.0).color.c[0];
        assert!(on_edge > 0.9, "the edge is near white, {on_edge}");
        assert!(on_line < interior, "the hairline is darker, {on_line}");
        assert!(
            (interior - from_left(0.1, 0.0).color.c[0]).abs() < 1.0e-6,
            "and the interior is flat"
        );
    }

    #[test]
    fn the_slab_unrolls_from_its_top_edge() {
        let at = |open: f32, y: f32| shade_at(open, 0.0, [0.5, y]).discarded();
        assert!(at(0.0, 0.5), "closed, nothing shows");
        assert!(!at(1.0, 0.5) && !at(1.0, 0.1), "open, all of it shows");
        assert!(!at(0.5, 0.9), "half open, the top half shows");
        assert!(at(0.5, 0.1), "and the bottom half not yet");
    }

    #[test]
    fn a_line_runs_along_the_front_only_while_it_moves() {
        let edge = style::FROST_EDGE_WIDTH * TALL;
        // Half open: the front is at mid height.
        let front = shade_at(0.5, 0.0, [0.5, 0.5 + edge / TALL / 4.0]).color.c[0];
        let well_behind = shade_at(0.5, 0.0, [0.5, 0.9]).color.c[0];
        assert!(front > well_behind, "the front is bright, {front}");
        let settled = shade_at(1.0, 0.0, [0.5, 0.5]).color.c[0];
        assert!((settled - well_behind).abs() < 1.0e-6, "open, no line");
    }
}
