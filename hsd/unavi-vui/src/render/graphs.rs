//! The shading VUI draws with.
//!
//! `wired:shading` compiles a graph once per distinct value and shares the
//! program across every prim that submits the same one, so a body just
//! submits [`shell`] to itself at creation time rather than being bound to a
//! hidden template prim. What differs between two motes is their overrides.

use wired_guest::math::{
    Color,
    Vec2,
    Vec3,
};

use crate::wired::{
    scene::document::{
        Document,
        Layer,
    },
    shading::graph::{
        BinaryOp,
        BlendMode,
        Combine2Op,
        Combine3Op,
        Combine4Op,
        ConvertOp,
        CullMode,
        ExtractOp,
        GraphValue,
        LerpOp,
        Node,
        Port,
        PowOp,
        RemapOp,
        ShaderGraph,
        SmoothstepOp,
        StepOp,
        SurfaceGraph,
        SurfaceOutput,
        UnlitOutput,
        ValueKind,
        set_graph,
        set_overrides,
    },
};

pub mod matrix;

/// Tint, with the body's alpha in its own alpha channel.
pub const SHELL_TINT: u16 = 0;
/// How brightly the shell burns from within, as a multiple of its own hue.
/// Carries what heat cannot: a toggle that is on holds a floor here whatever
/// attention it is under.
pub const SHELL_EMISSIVE: u16 = 1;
/// How far the mote has come toward the attention it is under.
pub const SHELL_HEAT: u16 = 2;
/// Radians of offset into the idle breath and the film's banding, so a form
/// does not pulse or shimmer as one body.
pub const SHELL_PHASE: u16 = 3;
/// How much of the iridescent film the shell wears, 0 for none. A mote that
/// opts in trades a stable colour for the moving hues of the bubble.
pub const SHELL_FILM: u16 = 4;
/// How much the shell's rim is frosted, 0 for clear glass. A mote that opts
/// in diffuses what shows through its edge rather than refracting it.
pub const SHELL_FROST: u16 = 5;
/// How far the mote has arrived, 0 for not yet and 1 for whole. Drawn as a
/// front sweeping up the body rather than as a fade, so a mote materializes
/// where it stands.
pub const SHELL_BLOOM: u16 = 6;

pub const RING_TINT: u16 = 0;
pub const RING_PROGRESS: u16 = 1;

/// The accent a pane's corner tab is drawn in; its alpha says whether there
/// is a tab at all.
pub const PANE_ACCENT: u16 = 0;
/// How many metres a pane's shape is drawn in per metre of its mote-scale
/// design, so a pane stood at several times that size keeps its proportions.
pub const PANE_UNIT: u16 = 1;

/// The frost laid over what a pane shows: cool, and light enough that dark
/// type reads on it over any room.
const FROST: Color = Color {
    r: 0.93,
    g: 0.96,
    b: 1.0,
    a: 1.0,
};
/// How far the blurred room is lifted toward [`FROST`]. Nearly all of the
/// way: dark type has to read on a pane over any backdrop, a black one
/// included.
const FROST_LIFT: f32 = 0.9;
/// Radius of the pane's rounded corners, in metres at mote scale.
const CORNER: f32 = 0.012;
/// How far along each edge the top-right corner is cut away, in metres at
/// mote scale.
const CHAMFER: f32 = 0.02;
/// The gap between the chamfered edge and the corner tab, measured square to
/// the edge, in metres at mote scale.
const TAB_GAP: f32 = 0.003;
/// How far the rim line is mixed toward white.
const RIM_MIX: f32 = 0.5;
/// How far the blur's taps stray from the centre, in screen coordinates. The
/// diagonals sit at three quarters of it so the nine taps cover a disc.
const PANE_SPREAD: f32 = 0.006;
/// Where a pane's edge line starts and ends fading, in metres from its border
/// at mote scale.
const EDGE: (f32, f32) = (0.0007, 0.0015);

/// Appends nodes and hands back a port naming each, so a graph is written as
/// what it computes rather than as a list of indices to keep in step.
#[derive(Default)]
struct Net {
    nodes: Vec<Node>,
}

/// The ports a shell's carried light is composited from: the room behind it,
/// the glass terms, and the attention driving it.
#[derive(Clone, Copy)]
struct Carried {
    power:  Port,
    rim:    Port,
    phase:  Port,
    gate:   Port,
    behind: Port,
    tint:   Port,
    heat:   Port,
}

impl Net {
    fn push(&mut self, node: Node) -> Port {
        self.nodes.push(node);
        // A network is capped far below this; the cast cannot be reached.
        Port::Node(u16::try_from(self.nodes.len() - 1).unwrap_or(u16::MAX))
    }

    /// A slow oscillator, offset per body so a form does not pulse as one
    /// object. Costs nothing per frame: the clock is the renderer's own.
    fn breath(&mut self, phase: Port, rate: f32) -> Port {
        let time = self.push(Node::Time);
        let scaled = self.push(Node::Mul(binary(time, cf(rate))));
        let offset = self.push(Node::Add(binary(scaled, phase)));
        self.push(Node::Sin(offset))
    }

    fn time_scaled(&mut self, rate: f32) -> Port {
        let time = self.push(Node::Time);
        self.push(Node::Mul(binary(time, cf(rate))))
    }

    /// Three sinusoids a third of a turn apart, which is a hue sweep without a
    /// colour ramp to sample: as `t` advances the triple walks the spectrum.
    fn spectrum(&mut self, t: Port) -> Port {
        let channel = |net: &mut Self, turn: f32| {
            let shifted = net.push(Node::Add(binary(t, cf(turn))));
            let wave = net.push(Node::Sin(shifted));
            net.push(Node::Remap(RemapOp {
                x:         wave,
                from_low:  cf(-1.0),
                from_high: cf(1.0),
                to_low:    cf(0.0),
                to_high:   cf(1.0),
            }))
        };
        let r = channel(self, 0.0);
        let g = channel(self, 2.094_395);
        let b = channel(self, 4.188_79);
        self.push(Node::Combine3(Combine3Op { x: r, y: g, z: b }))
    }

    /// How squarely the surface faces the viewer, `0` at the silhouette and
    /// `1` head-on. The complement of the term the rim is built from, and
    /// what everything view-dependent here is driven by.
    fn facing(&mut self) -> Port {
        let grazing = self.push(Node::Fresnel(cf(1.0)));
        self.push(Node::OneMinus(grazing))
    }

    /// Thin-film colour, as a bubble wears it.
    ///
    /// The hue stays near the mote's own colour rather than walking the whole
    /// wheel: the spectrum is pulled in only slightly, so the film reads as
    /// the same hue shimmering rather than as a different colour crawling
    /// across it. Red bends least and blue most, so the three rims sit at
    /// slightly different widths and the silhouette still separates into
    /// colour the way a curved film does. The banding drifts; `phase` keeps
    /// two motes from wearing the same pattern.
    fn interference(
        &mut self,
        power: Port,
        rim: Port,
        facing: Port,
        phase: Port,
        tint: Port,
        amount: Port,
    ) -> Port {
        let red_power = self.push(Node::Mul(binary(power, cf(0.78))));
        let blue_power = self.push(Node::Mul(binary(power, cf(1.34))));
        let red_rim = self.push(Node::Fresnel(red_power));
        let red_rim = self.push(Node::Saturate(red_rim));
        let blue_rim = self.push(Node::Fresnel(blue_power));
        let blue_rim = self.push(Node::Saturate(blue_rim));
        let split = self.push(Node::Combine3(Combine3Op {
            x: red_rim,
            y: rim,
            z: blue_rim,
        }));

        let tint_r = self.push(Node::Extract(ExtractOp {
            v:       tint,
            channel: 0,
        }));
        let tint_g = self.push(Node::Extract(ExtractOp {
            v:       tint,
            channel: 1,
        }));
        let tint_b = self.push(Node::Extract(ExtractOp {
            v:       tint,
            channel: 2,
        }));
        let tint_rgb = self.push(Node::Combine3(Combine3Op {
            x: tint_r,
            y: tint_g,
            z: tint_b,
        }));

        let grazing = self.push(Node::OneMinus(facing));
        let bands = self.push(Node::Mul(binary(grazing, cf(7.0))));
        let crawl = self.time_scaled(0.25);
        let bands = self.push(Node::Add(binary(bands, crawl)));
        let bands = self.push(Node::Add(binary(bands, phase)));
        let hues = self.spectrum(bands);
        // Mostly the tint, a little of the walk: the hue stays in a nearby
        // range rather than sweeping the wheel.
        let local = self.push(Node::Lerp(LerpOp {
            a: tint_rgb,
            b: hues,
            t: cf(0.2),
        }));
        let colored = self.push(Node::Mul(binary(local, split)));
        self.push(Node::Mul(binary(colored, amount)))
    }

    /// A tight highlight from a fixed direction. A mote does not turn, so this
    /// slides across it as the viewer moves — which is what makes a surface
    /// read as curved and glassy rather than as a flat disc.
    fn streak(&mut self, sharpness: f32) -> Port {
        let normal = self.push(Node::WorldNormal);
        let toward = self.push(Node::Dot(binary(
            normal,
            Port::Const(GraphValue::Vec3(Vec3::new(0.32, 0.79, 0.52))),
        )));
        let lit = self.push(Node::Saturate(toward));
        self.push(Node::Pow(PowOp {
            x: lit,
            y: cf(sharpness),
        }))
    }

    /// Light the shell carries on top of the tinted room: the rim outline
    /// always, and the opt-in bubble — thin-film film, two highlights, and a
    /// caustic ring — gated by `SHELL_FILM`, all breathing softly.
    ///
    /// Added rather than multiplied through, the way a highlight sits on top
    /// of the glass it is reflected in. Gating the bubble keeps a mote that
    /// did not ask for it on a stable colour.
    fn carried(&mut self, c: Carried) -> Port {
        let Carried {
            power,
            rim,
            phase,
            gate,
            behind,
            tint,
            heat,
        } = c;

        // A rim that is always drawn, and lights up with attention so a
        // selected mote glows at its silhouette rather than washing its
        // colour grey.
        let rest = self.push(Node::Mul(binary(rim, cf(0.05))));
        let lit = self.push(Node::Mul(binary(rim, heat)));
        let lit = self.push(Node::Mul(binary(lit, cf(0.18))));
        let outline = self.push(Node::Add(binary(rest, lit)));

        let facing = self.facing();
        let film = self.interference(power, rim, facing, phase, tint, cf(0.4));
        let film = self.push(Node::Mul(binary(film, Port::Input(SHELL_FILM))));
        let streak = self.streak(46.0);
        let streak = self.push(Node::Mul(binary(streak, Port::Input(SHELL_FILM))));
        let streak2 = self.streak(18.0);
        let streak2 = self.push(Node::Mul(binary(streak2, Port::Input(SHELL_FILM))));

        let glints = self.push(Node::Add(binary(outline, film)));
        let glints = self.push(Node::Add(binary(glints, streak)));
        let glints = self.push(Node::Add(binary(glints, streak2)));
        let glints = self.push(Node::Convert(ConvertOp {
            v:  glints,
            to: ValueKind::Color,
        }));

        // The caustic ring: light squeezed into a ring just inside the
        // silhouette, where the room behind is bent most. Gated with the
        // film, so a clear shell bends the room without adding a band of its
        // own.
        let caustic = self.push(Node::Mul(binary(gate, cf(0.15))));
        let caustic = self.push(Node::Mul(binary(caustic, Port::Input(SHELL_FILM))));
        let caustic = self.push(Node::Mul(binary(caustic, behind)));
        let glints = self.push(Node::Add(binary(glints, caustic)));

        // A soft idle breath in the carried light, not the room: the shell
        // shimmers as a form would, while what is behind it stays still.
        let wave = self.breath(phase, 1.6);
        let swing = self.push(Node::Mul(binary(wave, cf(0.1))));
        let breath = self.push(Node::Add(binary(swing, cf(1.0))));
        self.push(Node::Mul(binary(glints, breath)))
    }

    /// A rim-weighted multi-tap blur of the room behind, so a frosted shell
    /// diffuses its edge while the face stays a clear window.
    ///
    /// The blur follows the bent sample and is gated by the rim and by
    /// `SHELL_FROST`, so a shell that did not ask for frost keeps its clear
    /// refraction.
    fn frost(&mut self, at: Port, center: Port, rim: Port) -> Port {
        let spread = 0.005;
        let mut sum = center;
        for (dx, dy) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
            let tap_at = self.push(Node::Add(binary(
                at,
                Port::Const(GraphValue::Vec2(Vec2::new(dx * spread, dy * spread))),
            )));
            let tap = self.push(Node::SceneColor(tap_at));
            sum = self.push(Node::Add(binary(sum, tap)));
        }
        let blurred = self.push(Node::Mul(binary(sum, cf(0.2))));
        let amount = self.push(Node::Mul(binary(rim, Port::Input(SHELL_FROST))));
        self.push(Node::Lerp(LerpOp {
            a: center,
            b: blurred,
            t: amount,
        }))
    }

    /// Puts the body's own latitude in the alpha, which the terminal's
    /// threshold then cuts against — a front sweeping up the mote as it
    /// arrives, so it materializes where it stands rather than inflating or
    /// fading in.
    fn latitude_clipped(&mut self, color: Port) -> Port {
        let uv = self.push(Node::Uv);
        let up = self.push(Node::Extract(ExtractOp {
            v:       uv,
            channel: 1,
        }));
        let below = self.push(Node::OneMinus(up));
        with_alpha(self, color, below)
    }
}

const fn cf(v: f32) -> Port {
    Port::Const(GraphValue::Float(v))
}

const fn binary(a: Port, b: Port) -> BinaryOp {
    BinaryOp { a, b }
}

/// Splits a colour into channels and puts it back with `alpha`, which is the
/// only way to reach a terminal's alpha independently of its rgb.
fn with_alpha(net: &mut Net, color: Port, alpha: Port) -> Port {
    let r = net.push(Node::Extract(ExtractOp {
        v:       color,
        channel: 0,
    }));
    let g = net.push(Node::Extract(ExtractOp {
        v:       color,
        channel: 1,
    }));
    let b = net.push(Node::Extract(ExtractOp {
        v:       color,
        channel: 2,
    }));
    net.push(Node::Combine4(Combine4Op {
        x: r,
        y: g,
        z: b,
        w: alpha,
    }))
}

/// A mote's shell: a bubble whose transmission shows the room behind it and
/// whose edge carries it.
///
/// The refraction follows the mote's own curvature — the sample offset is the
/// surface normal's screen projection — so the face is a window and the room
/// bends only at the silhouette: a lens, not a flat pane. Over that comes the
/// hue, at the strength the glass alpha asks for; the interior lamp; and last
/// the light the shell carries, which is the rim always and the opt-in bubble
/// of film, highlights and caustic on top of it.
///
/// The material is opaque because that is what reaches the transmissive
/// phase: a blended one lands in the transparent phase, where the texture
/// `SceneColor` samples is never filled.
pub fn shell() -> ShaderGraph {
    let mut net = Net::default();
    let screen = net.push(Node::ScreenUv);
    let normal = net.push(Node::WorldNormal);
    let across = net.push(Node::Convert(ConvertOp {
        v:  normal,
        to: ValueKind::Vec2,
    }));
    // The projected distance from the mote's own centre: exactly 0 at the
    // face, exactly 1 at the silhouette. Refraction is confined to the outer
    // ring of that radius, so the face — where the mote's icon and contents
    // live — is a perfect window and only the silhouette bends the room.
    let radius = net.push(Node::Length(across));
    let gate = net.push(Node::Smoothstep(SmoothstepOp {
        low:  cf(0.75),
        high: cf(0.98),
        x:    radius,
    }));
    let amount = net.push(Node::Mul(binary(gate, cf(0.04))));
    let bend = net.push(Node::Mul(binary(across, amount)));
    let at = net.push(Node::Add(binary(screen, bend)));
    let behind = net.push(Node::SceneColor(at));

    // The edge is what carries a mote, and it narrows as heat rises so an
    // attended mote does not merely brighten — its edge thickens, which still
    // reads at a distance where a brightness change does not. The rim is
    // always on: it is the outline that keeps a shell readable even when its
    // glass is clear.
    let heat = Port::Input(SHELL_HEAT);
    let phase = Port::Input(SHELL_PHASE);
    let power = net.push(Node::Lerp(LerpOp {
        a: cf(3.0),
        b: cf(1.2),
        t: heat,
    }));
    let rim = net.push(Node::Fresnel(power));
    let rim = net.push(Node::Saturate(rim));

    // A frosted shell diffuses what shows through its rim: the room is
    // blurred where the silhouette bends it, and the face stays a clear
    // window so the icon is never lost behind the frost.
    let behind = net.frost(at, behind, rim);

    // Coloured glass mixes the room behind with the mote's own hue. The tint's
    // alpha is the glass strength, and the hue is mixed in as light the shell
    // carries rather than as a filter over the room — so a bright room does
    // not wash a saturated tint out, and the shell reads as lit from within.
    let strength = net.push(Node::Extract(ExtractOp {
        v:       Port::Input(SHELL_TINT),
        channel: 3,
    }));
    let r = net.push(Node::Extract(ExtractOp {
        v:       Port::Input(SHELL_TINT),
        channel: 0,
    }));
    let g = net.push(Node::Extract(ExtractOp {
        v:       Port::Input(SHELL_TINT),
        channel: 1,
    }));
    let b = net.push(Node::Extract(ExtractOp {
        v:       Port::Input(SHELL_TINT),
        channel: 2,
    }));
    let hue = net.push(Node::Combine4(Combine4Op {
        x: r,
        y: g,
        z: b,
        w: cf(1.0),
    }));
    let glass = net.push(Node::Lerp(LerpOp {
        a: behind,
        b: hue,
        t: strength,
    }));

    // The interior lamp: the shell's own hue, at the brightness the palette
    // asked for. Separate from the carried light because it is the volume
    // burning rather than the surface catching something, and because it says
    // what heat cannot — a toggle that is on burns whether or not anything is
    // looking at it.
    let lit = net.push(Node::Mul(binary(hue, Port::Input(SHELL_EMISSIVE))));

    let carried = net.carried(Carried {
        power,
        rim,
        phase,
        gate,
        behind,
        tint: Port::Input(SHELL_TINT),
        heat,
    });
    let color = net.push(Node::Add(binary(glass, carried)));
    let color = net.push(Node::Add(binary(color, lit)));

    // Clipping is the only way to reach `discard` from a graph, and it
    // composes with the opaque blend rather than replacing it — so a mote can
    // arrive without the shell leaving the phase its transmission is drawn in.
    let front = net.push(Node::OneMinus(Port::Input(SHELL_BLOOM)));
    let color = net.latitude_clipped(color);

    ShaderGraph {
        public_inputs: vec![
            GraphValue::Color(Color {
                r: 0.92,
                g: 0.94,
                b: 0.96,
                a: 0.16,
            }),
            GraphValue::Float(0.08),
            GraphValue::Float(0.0),
            GraphValue::Float(0.0),
            GraphValue::Float(crate::palette::FILM),
            GraphValue::Float(0.0),
            GraphValue::Float(1.0),
        ],
        surface:       SurfaceGraph {
            nodes:        net.nodes,
            output:       SurfaceOutput::Unlit(UnlitOutput {
                color,
                alpha_clip_threshold: Some(front),
            }),
            blend:        BlendMode::Opaque,
            cull:         CullMode::Back,
            cast_shadows: false,
        },
        displacement:  None,
    }
}

/// A cast ring, filled by a sweep rather than by growing.
///
/// Growing conflates how far along a cast is with how big it is; a sweep says
/// only the first, and the burning head says which way it is going.
pub fn ring() -> ShaderGraph {
    let mut net = Net::default();
    let progress = Port::Input(RING_PROGRESS);

    let uv = net.push(Node::Uv);
    let angle = net.push(Node::Extract(ExtractOp {
        v:       uv,
        channel: 0,
    }));
    let across = net.push(Node::Extract(ExtractOp {
        v:       uv,
        channel: 1,
    }));

    // The fill's leading edge is eased over a slice of the turn rather than
    // stepped, so the boundary is a boundary and not a cut.
    let lead = net.push(Node::Add(binary(angle, cf(0.035))));
    let filled = net.push(Node::Smoothstep(SmoothstepOp {
        low:  angle,
        high: lead,
        x:    progress,
    }));

    // A short tail behind the head, which is what makes a fill read as
    // travelling rather than as a bar growing.
    let behind = net.push(Node::Sub(binary(progress, angle)));
    let behind = net.push(Node::Saturate(behind));
    let behind = net.push(Node::OneMinus(behind));
    let head = net.push(Node::Pow(PowOp {
        x: behind,
        y: cf(18.0),
    }));
    let head = net.push(Node::Mul(binary(head, filled)));
    let head = net.push(Node::Mul(binary(head, cf(0.8))));

    // The unfilled arc stays faintly drawn: a cast shows how far it has to go,
    // not only how far it has come.
    let body = net.push(Node::Mul(binary(filled, cf(0.7))));
    let body = net.push(Node::Add(binary(body, cf(0.12))));
    let total = net.push(Node::Add(binary(body, head)));

    // A soft falloff across the whole band rather than a flat core with eased
    // sides: the ring has no hard edge anywhere, which is what stops a thin
    // bright annulus reading as a cut-out.
    let centred = net.push(Node::Sub(binary(across, cf(0.5))));
    let centred = net.push(Node::Abs(centred));
    // Rising edges only: a smoothstep with its edges the wrong way round is
    // undefined, so the falloff is built rising and then inverted.
    let rising = net.push(Node::Smoothstep(SmoothstepOp {
        low:  cf(0.0),
        high: cf(0.5),
        x:    centred,
    }));
    let falloff = net.push(Node::OneMinus(rising));
    let band = net.push(Node::Mul(binary(falloff, falloff)));
    let total = net.push(Node::Mul(binary(total, band)));
    let total = net.push(Node::Mul(binary(total, cf(2.2))));

    let rgb = net.push(Node::Mul(binary(Port::Input(RING_TINT), total)));
    // Alpha 0, which is what additive means here: `AlphaMode::Add` blends
    // `src.rgb + dst * (1 - src.a)`, so an alpha of 1 would erase whatever the
    // ring is drawn over and leave its faded edges reading as black rather
    // than as nothing.
    let color = with_alpha(&mut net, rgb, cf(0.0));

    ShaderGraph {
        public_inputs: vec![
            GraphValue::Color(Color {
                r: 0.60,
                g: 0.84,
                b: 1.00,
                a: 1.0,
            }),
            GraphValue::Float(0.0),
        ],
        surface:       SurfaceGraph {
            nodes:        net.nodes,
            output:       SurfaceOutput::Unlit(UnlitOutput {
                color,
                alpha_clip_threshold: None,
            }),
            blend:        BlendMode::Add,
            cull:         CullMode::None,
            cast_shadows: false,
        },
        displacement:  None,
    }
}

impl Net {
    /// The room behind a pane, blurred over nine taps and lifted nearly to
    /// the frost.
    fn frosted_room(&mut self) -> Port {
        let screen = self.push(Node::ScreenUv);
        let mut sum = self.push(Node::SceneColor(screen));
        let diagonal = PANE_SPREAD * 0.75;
        let taps = [
            (PANE_SPREAD, 0.0),
            (-PANE_SPREAD, 0.0),
            (0.0, PANE_SPREAD),
            (0.0, -PANE_SPREAD),
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
        self.push(Node::Lerp(LerpOp {
            a: blurred,
            b: Port::Const(GraphValue::Color(FROST)),
            t: cf(FROST_LIFT),
        }))
    }
}

/// A pane of frosted glass: the room behind it, blurred and lifted nearly to a
/// cool white, with a faint bright line on its border. Its corners are
/// rounded, but for the top right, which is cut away on the diagonal and
/// holds a small solid triangle in the placard's accent colour.
///
/// The shape is an alpha clip on a flat quad rather than geometry. Like the
/// shell it is opaque so it reaches the phase `SceneColor` is filled in,
/// unlit so the lit transmissive path cannot occlude it, and arrives by
/// scale. It is double sided so a placard reads from whichever side the
/// viewer stands.
pub fn pane() -> ShaderGraph {
    let mut net = Net::default();
    let frosted = net.frosted_room();

    // Every length below is in metres, so the shape is the same on a tall pane
    // as on a wide one: the quad is stretched, its uv is not. `unit` carries a
    // pane stood at several times mote scale.
    let unit = Port::Input(PANE_UNIT);
    let scaled = |net: &mut Net, metres: f32| net.push(Node::Mul(binary(unit, cf(metres))));
    let corner = scaled(&mut net, CORNER);
    let chamfer = scaled(&mut net, CHAMFER);
    let gap = scaled(&mut net, TAB_GAP);
    let edge_low = scaled(&mut net, EDGE.0);
    let edge_high = scaled(&mut net, EDGE.1);

    // How far a fragment is from the low and the high border along an axis.
    let uv = net.push(Node::Uv);
    let scale = net.push(Node::ObjectScale);
    let axis = |net: &mut Net, channel: u8| {
        let across = net.push(Node::Extract(ExtractOp { v: uv, channel }));
        let back = net.push(Node::OneMinus(across));
        let size = net.push(Node::Extract(ExtractOp { v: scale, channel }));
        let low = net.push(Node::Mul(binary(across, size)));
        let high = net.push(Node::Mul(binary(back, size)));
        (low, high)
    };
    let (left, right) = axis(&mut net, 0);
    let (bottom, top) = axis(&mut net, 1);
    let wide = net.push(Node::Min(binary(left, right)));
    let tall = net.push(Node::Min(binary(bottom, top)));

    // Depth inside a rounded rectangle: the nearer border's distance, except
    // within a corner's radius of both, where it is the radius less the
    // distance from the corner circle's centre.
    let straight = net.push(Node::Min(binary(wide, tall)));
    let round_axis = |net: &mut Net, depth: Port| {
        let short = net.push(Node::Sub(binary(corner, depth)));
        net.push(Node::Max(binary(short, cf(0.0))))
    };
    let round_wide = round_axis(&mut net, wide);
    let round_tall = round_axis(&mut net, tall);
    let corner_at = net.push(Node::Combine2(Combine2Op {
        x: round_wide,
        y: round_tall,
    }));
    let reach = net.push(Node::Length(corner_at));
    let rounded = net.push(Node::Sub(binary(corner, reach)));
    let boxed = net.push(Node::Min(binary(straight, rounded)));

    // The top-right corner is cut along a diagonal instead of rounded: signed
    // metres from that line, negative on the corner's side of it.
    let legs = net.push(Node::Add(binary(right, top)));
    let past = net.push(Node::Sub(binary(legs, chamfer)));
    let cut = net.push(Node::Mul(binary(past, cf(std::f32::consts::FRAC_1_SQRT_2))));
    let depth = net.push(Node::Min(binary(boxed, cut)));
    let body = net.push(Node::Step(StepOp {
        edge: cf(0.0),
        x:    depth,
    }));

    // A faint bright rim, where the pane's own edge is.
    let inside = net.push(Node::Smoothstep(SmoothstepOp {
        low:  edge_low,
        high: edge_high,
        x:    depth,
    }));
    let edge = net.push(Node::OneMinus(inside));
    let rim = net.push(Node::Mul(binary(edge, cf(RIM_MIX))));
    let line = net.push(Node::Lerp(LerpOp {
        a: frosted,
        b: Port::Const(GraphValue::Color(Color::WHITE)),
        t: rim,
    }));

    // The corner tab: what the chamfer took off, less a gap, in the accent.
    // An accent with no alpha has no tab, and is clipped like the rest.
    let beyond = net.push(Node::Sub(binary(cf(0.0), cut)));
    let in_tab = net.push(Node::Step(StepOp {
        edge: gap,
        x:    beyond,
    }));
    let accent = Port::Input(PANE_ACCENT);
    let accent_alpha = net.push(Node::Extract(ExtractOp {
        v:       accent,
        channel: 3,
    }));
    let tab = net.push(Node::Mul(binary(in_tab, accent_alpha)));
    let painted = net.push(Node::Lerp(LerpOp {
        a: line,
        b: accent,
        t: tab,
    }));
    let covered = net.push(Node::Max(binary(body, tab)));
    let color = with_alpha(&mut net, painted, covered);

    ShaderGraph {
        public_inputs: vec![
            GraphValue::Color(accent_input(None)),
            GraphValue::Float(1.0),
        ],
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

/// Submits [`shell`] to `prim`. Cheap and repeatable: the host shares one
/// compiled program across every prim submitting the same graph, so this is
/// paid once per slot rather than built into a hidden template.
pub fn set_shell(doc: &Document, prim: (u64, u64)) -> anyhow::Result<()> {
    set_graph(doc, prim, Layer::Local, Some(&shell()))?;
    Ok(())
}

pub fn set_ring(doc: &Document, prim: (u64, u64)) -> anyhow::Result<()> {
    set_graph(doc, prim, Layer::Local, Some(&ring()))?;
    Ok(())
}

pub fn set_pane(doc: &Document, prim: (u64, u64)) -> anyhow::Result<()> {
    set_graph(doc, prim, Layer::Local, Some(&pane()))?;
    Ok(())
}

/// What a pane is handed per prim: its accent, if it has one, and the scale
/// its shape is drawn at.
pub fn set_pane_look(
    doc: &Document,
    prim: (u64, u64),
    accent: Option<Color>,
    unit: f32,
) -> anyhow::Result<()> {
    set_overrides(
        doc,
        prim,
        Layer::Local,
        &[
            (PANE_ACCENT, GraphValue::Color(accent_input(accent))),
            (PANE_UNIT, GraphValue::Float(unit)),
        ],
    )?;
    Ok(())
}

/// The accent as the pane reads it: whole alpha for a colour, none for no tab.
const fn accent_input(accent: Option<Color>) -> Color {
    match accent {
        Some(color) => Color { a: 1.0, ..color },
        None => Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    /// `hsd::attributes::shader::MAX_NODES`, restated because a guest
    /// cannot see it: exceeding it is refused at the setter, so the whole
    /// surface would come up unshaded.
    const MAX_NODES: usize = 128;
    const MAX_PUBLIC_INPUTS: usize = 16;

    fn ports(node: &Node) -> Vec<Port> {
        match node {
            Node::Uv
            | Node::ScreenUv
            | Node::WorldNormal
            | Node::WorldPosition
            | Node::VertexColor
            | Node::LocalPosition
            | Node::LocalNormal
            | Node::Time
            | Node::InstanceRandom
            | Node::ObjectPosition
            | Node::ObjectScale
            | Node::ViewDirection => Vec::new(),
            Node::Sin(p)
            | Node::Cos(p)
            | Node::OneMinus(p)
            | Node::Abs(p)
            | Node::Floor(p)
            | Node::Fract(p)
            | Node::Saturate(p)
            | Node::Sqrt(p)
            | Node::Length(p)
            | Node::Normalize(p)
            | Node::TriangleWave(p)
            | Node::Luminance(p)
            | Node::Fresnel(p)
            | Node::Noise(p) => vec![*p],
            Node::Add(op)
            | Node::Sub(op)
            | Node::Mul(op)
            | Node::Div(op)
            | Node::Modulo(op)
            | Node::Min(op)
            | Node::Max(op)
            | Node::Dot(op)
            | Node::Cross(op)
            | Node::Distance(op) => vec![op.a, op.b],
            Node::Pow(op) => vec![op.x, op.y],
            Node::Atan2(op) => vec![op.y, op.x],
            Node::Lerp(op) => vec![op.a, op.b, op.t],
            Node::Clamp(op) => vec![op.x, op.low, op.high],
            Node::Step(op) => vec![op.edge, op.x],
            Node::Smoothstep(op) => vec![op.low, op.high, op.x],
            Node::Remap(op) => vec![op.x, op.from_low, op.from_high, op.to_low, op.to_high],
            Node::Select(op) => vec![op.cond, op.a, op.b],
            Node::TextureSample(op) => vec![op.uv],
            Node::SceneColor(uv) => vec![*uv],
            Node::Extract(op) => vec![op.v],
            Node::Combine2(op) => vec![op.x, op.y],
            Node::Combine3(op) => vec![op.x, op.y, op.z],
            Node::Combine4(op) => vec![op.x, op.y, op.z, op.w],
            Node::Convert(op) => vec![op.v],
            Node::PolarCoords(op) => vec![op.uv, op.center],
            Node::RotateUv(op) => vec![op.uv, op.center, op.radians],
        }
    }

    /// The two rules a graph is refused for that a guest can check on its own.
    /// Kind agreement it cannot: only the host holds the type rules, and a
    /// mismatch shows up as the rejection the host returns.
    fn assert_well_formed(graph: &ShaderGraph, name: &str) {
        assert!(
            graph.public_inputs.len() <= MAX_PUBLIC_INPUTS,
            "{name} declares {} inputs",
            graph.public_inputs.len()
        );
        assert!(
            graph.surface.nodes.len() <= MAX_NODES,
            "{name} has {} surface nodes, over the cap of {MAX_NODES}",
            graph.surface.nodes.len()
        );

        for (index, node) in graph.surface.nodes.iter().enumerate() {
            for port in ports(node) {
                match port {
                    Port::Node(target) => assert!(
                        usize::from(target) < index,
                        "{name} node {index} reaches forward to {target}"
                    ),
                    Port::Input(input) => assert!(
                        usize::from(input) < graph.public_inputs.len(),
                        "{name} node {index} names input {input}, which is not declared"
                    ),
                    Port::Const(_) => {}
                }
            }
        }
    }

    #[test]
    fn the_shell_is_well_formed() {
        assert_well_formed(&shell(), "shell");
    }

    #[test]
    fn the_ring_is_well_formed() {
        assert_well_formed(&ring(), "ring");
    }

    #[test]
    fn the_pane_is_well_formed() {
        assert_well_formed(&pane(), "pane");
    }

    /// Frost is the room behind the pane, so it has to be sampled in the phase
    /// that fills it, and several times or it is not blurred at all.
    #[test]
    fn the_pane_blurs_the_room_in_the_transmissive_phase() {
        let pane = pane();
        assert!(matches!(pane.surface.blend, BlendMode::Opaque));
        assert!(matches!(pane.surface.output, SurfaceOutput::Unlit(_)));
        let taps = pane
            .surface
            .nodes
            .iter()
            .filter(|node| matches!(node, Node::SceneColor(_)))
            .count();
        assert!((5..=9).contains(&taps), "{taps} taps");
    }

    #[test]
    fn the_pane_border_is_a_white_rim() {
        let rim = pane().surface.nodes.iter().any(|node| {
            matches!(
                node,
                Node::Lerp(LerpOp { b: Port::Const(GraphValue::Color(color)), .. })
                    if *color == Color::WHITE
            )
        });
        assert!(rim, "the line is mixed toward white");
    }

    /// The corners are cut, not drawn, so the pane has to clip, and to do it
    /// without leaving the phase its frost is sampled in.
    #[test]
    fn the_pane_cuts_its_shape_by_clipping_and_stays_opaque() {
        let pane = pane();
        assert!(matches!(pane.surface.blend, BlendMode::Opaque));
        let SurfaceOutput::Unlit(unlit) = pane.surface.output else {
            panic!("the pane is unlit");
        };
        assert!(unlit.alpha_clip_threshold.is_some());
    }

    /// The shape is measured in metres through the object's scale, not in
    /// uv, which would stretch the corner with the pane.
    #[test]
    fn the_pane_measures_its_shape_in_metres() {
        let nodes = pane().surface.nodes;
        assert!(nodes.iter().any(|node| matches!(node, Node::ObjectScale)));
        assert!(nodes.iter().any(|node| matches!(node, Node::Uv)));
    }

    #[test]
    fn the_pane_takes_an_accent_and_a_unit_per_prim() {
        let inputs = pane().public_inputs;
        assert_eq!(inputs.len(), 2);
        assert!(matches!(
            inputs[usize::from(PANE_ACCENT)],
            GraphValue::Color(Color { a, .. }) if a.abs() < f32::EPSILON
        ));
        assert!(matches!(
            inputs[usize::from(PANE_UNIT)],
            GraphValue::Float(unit) if (unit - 1.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn no_accent_means_no_tab_and_an_accent_is_whole() {
        assert!(accent_input(None).a.abs() < f32::EPSILON);
        let accent = accent_input(Some(Color::rgba(0.2, 0.4, 0.8, 0.3)));
        assert_eq!((accent.r, accent.g, accent.b), (0.2, 0.4, 0.8));
        assert!((accent.a - 1.0).abs() < f32::EPSILON);
    }

    /// The tab sits inside the cut-away triangle with room to spare, or the
    /// gap would eat it.
    #[test]
    fn the_tab_has_room_inside_the_chamfer() {
        let leg = TAB_GAP.mul_add(-std::f32::consts::SQRT_2, CHAMFER);
        assert!(leg > CHAMFER / 2.0, "a tab leg of {leg} m is a speck");
        assert!(CORNER < CHAMFER);
    }

    /// Luminance, reading the colour as the linear value the shader works in:
    /// a pessimistic reading, since a gamma encoding would only be darker.
    fn luminance(color: Color) -> f32 {
        0.0722f32.mul_add(color.b, 0.2126f32.mul_add(color.r, 0.7152 * color.g))
    }

    fn contrast(a: f32, b: f32) -> f32 {
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    /// The pane over the worst backdrop there is, black: the room adds
    /// nothing and only the frost is left.
    fn darkest_pane() -> f32 {
        FROST_LIFT * luminance(FROST)
    }

    #[test]
    fn every_ink_reads_on_the_pane_over_any_backdrop() {
        let palette = crate::palette::Palette::DEFAULT;
        for (name, ink) in [
            ("ink", palette.ink),
            ("muted", palette.muted_ink),
            ("warning", palette.warning),
        ] {
            let ratio = contrast(darkest_pane(), luminance(ink));
            assert!(ratio >= 4.5, "{name} ink is {ratio}:1 on the darkest pane");
        }
        assert!(
            luminance(palette.muted_ink) > luminance(palette.ink),
            "muted is quieter than ink"
        );
    }

    #[test]
    fn the_matrix_is_well_formed() {
        assert_well_formed(&matrix::matrix(), "matrix");
    }

    /// Additive and double sided: the face adds light to the glass behind it,
    /// and is seen from whichever side the viewer comes to.
    #[test]
    fn the_matrix_adds_light_and_draws_from_both_sides() {
        let matrix = matrix::matrix();
        assert!(matches!(matrix.surface.blend, BlendMode::Add));
        assert!(matches!(matrix.surface.cull, CullMode::None));
    }

    /// Arriving is a clip, not a fade: a shell that faded would have to
    /// blend, and a blended shell never reaches the phase its transmission
    /// is filled in.
    #[test]
    fn the_shell_clips_as_it_arrives_and_stays_opaque() {
        let shell = shell();
        assert!(matches!(shell.surface.blend, BlendMode::Opaque));

        let SurfaceOutput::Unlit(unlit) = shell.surface.output else {
            panic!("the shell is unlit");
        };
        assert!(
            unlit.alpha_clip_threshold.is_some(),
            "without a threshold the bloom input is carried and never drawn"
        );
    }

    /// The other direction: an input no node reads is an override pushed for
    /// nothing, and the side pushing it cannot see that it is dead. Terminals
    /// are not scanned — an input worth declaring is one something computes
    /// with.
    #[test]
    fn every_declared_input_is_read() {
        for (graph, name) in [
            (shell(), "shell"),
            (ring(), "ring"),
            (pane(), "pane"),
            (matrix::matrix(), "matrix"),
        ] {
            let networks = graph.surface.nodes.iter().chain(
                graph
                    .displacement
                    .iter()
                    .flat_map(|network| network.nodes.iter()),
            );
            let read = networks
                .flat_map(ports)
                .filter_map(|port| match port {
                    Port::Input(input) => Some(usize::from(input)),
                    Port::Node(_) | Port::Const(_) => None,
                })
                .collect::<HashSet<_>>();

            for input in 0..graph.public_inputs.len() {
                assert!(
                    read.contains(&input),
                    "{name} declares input {input}, and nothing reads it"
                );
            }
        }
    }

    /// Every input the shell is handed at draw time has to exist, or the
    /// override is refused and the mote keeps the graph's own defaults.
    #[test]
    fn every_shell_input_the_binding_writes_is_declared() {
        let count = shell().public_inputs.len();
        for input in [
            SHELL_TINT,
            SHELL_EMISSIVE,
            SHELL_HEAT,
            SHELL_PHASE,
            SHELL_FILM,
            SHELL_FROST,
            SHELL_BLOOM,
        ] {
            assert!(usize::from(input) < count, "input {input} is not declared");
        }
    }

    #[test]
    fn every_ring_input_the_binding_writes_is_declared() {
        let count = ring().public_inputs.len();
        for input in [RING_TINT, RING_PROGRESS] {
            assert!(usize::from(input) < count, "input {input} is not declared");
        }
    }
}
