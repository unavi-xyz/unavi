//! The node builder the shell and ring graphs are written with.

use wired_guest::math::{
    Vec2,
    Vec3,
};

use super::{
    ARRIVAL_FRONT,
    SHELL_FILM,
    SHELL_FROST,
};
use crate::wired::shading::graph::{
    BinaryOp,
    Combine3Op,
    Combine4Op,
    ConvertOp,
    DitherOp,
    ExtractOp,
    GraphValue,
    LerpOp,
    Node,
    Port,
    PowOp,
    RemapOp,
    ValueKind,
};

/// Appends nodes and hands back a port naming each, so a graph is written as
/// what it computes rather than as a list of indices to keep in step.
#[derive(Default)]
pub(super) struct Net {
    pub(super) nodes: Vec<Node>,
}

/// The ports a shell's carried light is composited from: the room behind it,
/// the glass terms, and the attention driving it.
#[derive(Clone, Copy)]
pub(super) struct Carried {
    pub(super) power:  Port,
    pub(super) rim:    Port,
    pub(super) phase:  Port,
    pub(super) gate:   Port,
    pub(super) behind: Port,
    pub(super) tint:   Port,
    pub(super) heat:   Port,
}

impl Net {
    pub(super) fn push(&mut self, node: Node) -> Port {
        self.nodes.push(node);
        // A network is capped far below this; the cast cannot be reached.
        Port::Node(u16::try_from(self.nodes.len() - 1).unwrap_or(u16::MAX))
    }

    /// A slow oscillator, offset per body so a form does not pulse as one
    /// object. Costs nothing per frame: the clock is the renderer's own.
    pub(super) fn breath(&mut self, phase: Port, rate: f32) -> Port {
        let time = self.push(Node::Time);
        let scaled = self.push(Node::Mul(binary(time, cf(rate))));
        let offset = self.push(Node::Add(binary(scaled, phase)));
        self.push(Node::Sin(offset))
    }

    pub(super) fn time_scaled(&mut self, rate: f32) -> Port {
        let time = self.push(Node::Time);
        self.push(Node::Mul(binary(time, cf(rate))))
    }

    /// Three sinusoids a third of a turn apart, which is a hue sweep without a
    /// colour ramp to sample: as `t` advances the triple walks the spectrum.
    pub(super) fn spectrum(&mut self, t: Port) -> Port {
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
    pub(super) fn facing(&mut self) -> Port {
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
    pub(super) fn interference(
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
    pub(super) fn streak(&mut self, sharpness: f32) -> Port {
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
    pub(super) fn carried(&mut self, c: Carried) -> Port {
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
    pub(super) fn frost(&mut self, at: Port, center: Port, rim: Port) -> Port {
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

    /// Puts a dot threshold in the alpha, which the terminal's clip then cuts
    /// against: a front sweeping up the body as `bloom` rises from 0 to 1,
    /// whose edge dissolves in the 4×4 Bayer pattern over a band
    /// `ARRIVAL_FRONT` tall. The mote materializes where it stands rather
    /// than inflating or fading in.
    pub(super) fn dissolved(&mut self, color: Port, bloom: Port) -> Port {
        let uv = self.push(Node::Uv);
        let up = self.push(Node::Extract(ExtractOp {
            v:       uv,
            channel: 1,
        }));
        let reach = self.push(Node::Mul(binary(bloom, cf(1.0 + ARRIVAL_FRONT))));
        let past = self.push(Node::Sub(binary(reach, up)));
        let level = self.push(Node::Mul(binary(past, cf(1.0 / ARRIVAL_FRONT))));
        let dots = self.push(Node::ScreenDots);
        let kept = self.push(Node::Dither(DitherOp { coord: dots, level }));
        with_alpha(self, color, kept)
    }
}

pub(super) const fn cf(v: f32) -> Port {
    Port::Const(GraphValue::Float(v))
}

pub(super) const fn binary(a: Port, b: Port) -> BinaryOp {
    BinaryOp { a, b }
}

/// Splits a colour into channels and puts it back with `alpha`, which is the
/// only way to reach a terminal's alpha independently of its rgb.
pub(super) fn with_alpha(net: &mut Net, color: Port, alpha: Port) -> Port {
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
