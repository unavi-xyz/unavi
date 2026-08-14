//! The shading VUI draws with, built once and bound by every body that wants
//! it.
//!
//! A graph is submitted to one hidden template prim and reached by
//! `material:binding`, rather than written onto each body: binding costs
//! nothing, where submitting costs an upload, and the renderer shares one
//! compiled program across everything pointing at it. What differs between two
//! motes is their overrides.

use std::cell::RefCell;

use wired_prelude::prelude::*;

use crate::wired::scene::{
    api::self_document,
    types::{
        BlendMode,
        Combine3Op,
        Combine4Op,
        ConvertOp,
        CullMode,
        Document,
        ExtractOp,
        GraphValue,
        LerpOp,
        Node,
        Port,
        PowOp,
        Prim,
        RemapOp,
        ShaderGraph,
        SmoothstepOp,
        SurfaceGraph,
        SurfaceOutput,
        UnlitOutput,
        ValueKind,
        Xform,
    },
};

/// Tint, with the body's alpha in its own alpha channel.
pub const SHELL_TINT: u16 = 0;
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

pub const RING_TINT: u16 = 0;
pub const RING_PROGRESS: u16 = 1;

/// Appends nodes and hands back a port naming each, so a graph is written as
/// what it computes rather than as a list of indices to keep in step.
#[derive(Default)]
struct Net {
    nodes: Vec<Node>,
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
    fn carried(
        &mut self,
        power: Port,
        rim: Port,
        phase: Port,
        gate: Port,
        behind: Port,
        tint: Port,
        heat: Port,
    ) -> Port {
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
}

const fn cf(v: f32) -> Port {
    Port::Const(GraphValue::Float(v))
}

const fn binary(a: Port, b: Port) -> crate::wired::scene::types::BinaryOp {
    crate::wired::scene::types::BinaryOp { a, b }
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
/// Step 1 proved the transmission pipeline — the motes show the room behind
/// them. Step 2 makes that refraction follow the mote's own curvature: the
/// sample offset is the surface normal's screen projection, scaled by the
/// squared rim, so the face is a window (normal ≈ straight ahead) and the
/// world behind bends hardest from halfway out to the silhouette — a lens,
/// not a flat pane. Step 3 tints the sampled room with the mote's hue, the
/// strength following the glass alpha, so an idle mote's glass is nearly
/// clear and an attended one wears its colour. Step 4 adds light the shell
/// carries — a rim outline always, and on top of it the opt-in film:
/// rim-weighted thin-film interference, two highlights, and a caustic ring,
/// gated by `SHELL_FILM` so a mote wears a stable colour unless it asks for
/// the bubble. Step 5 makes the rim optional frost: a rim-weighted multi-tap
/// blur of the room, gated by `SHELL_FROST` so a shell that did not ask for
/// it keeps its clear refraction.
fn shell() -> ShaderGraph {
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

    let carried = net.carried(
        power,
        rim,
        phase,
        gate,
        behind,
        Port::Input(SHELL_TINT),
        heat,
    );
    let color = net.push(Node::Add(binary(glass, carried)));

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
        ],
        surface:       SurfaceGraph {
            nodes:        net.nodes,
            output:       SurfaceOutput::Unlit(UnlitOutput {
                color,
                alpha_clip_threshold: None,
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
fn ring() -> ShaderGraph {
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
    let falloff = net.push(Node::Smoothstep(SmoothstepOp {
        low:  cf(0.5),
        high: cf(0.0),
        x:    centred,
    }));
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

/// The prims carrying VUI's compiled graphs, minted once for the document
/// every surface draws into.
struct Templates {
    shell: Prim,
    ring:  Prim,
}

thread_local! {
    static TEMPLATES: RefCell<Option<Templates>> = const { RefCell::new(None) };
}

fn template(doc: &Document, name: &str, graph: &ShaderGraph) -> anyhow::Result<Prim> {
    let prim = doc.create_prim()?;
    // Carries a graph and nothing else: with no mesh there is nothing to draw,
    // and the hidden xform keeps it from being mistaken for content.
    prim.set_xform(Some(Xform {
        translation: Vec3::ZERO,
        rotation:    Quat::IDENTITY,
        scale:       Vec3::ZERO,
    }))?;
    // Reported rather than propagated: the host names the node it rejected,
    // and a surface that comes up unshaded with that on the log is far easier
    // to place than one that refuses to come up at all.
    if let Err(err) = prim.set_material_graph(Some(graph)) {
        eprintln!("vui: the {name} graph was rejected, so it draws unshaded: {err}");
    }
    Ok(prim)
}

fn with_templates<T>(f: impl FnOnce(&Templates) -> T) -> anyhow::Result<T> {
    TEMPLATES.with(|cell| {
        let mut slot = cell.borrow_mut();
        if slot.is_none() {
            let doc = self_document()?;
            *slot = Some(Templates {
                shell: template(&doc, "shell", &shell())?,
                ring:  template(&doc, "ring", &ring())?,
            });
        }
        let templates = slot.as_ref().expect("just filled");
        Ok(f(templates))
    })
}

/// Points `prim` at the shell graph. Cheap and repeatable: a binding names a
/// prim, and every body naming this one shares its compiled program.
pub fn bind_shell(prim: &Prim) -> anyhow::Result<()> {
    let id = with_templates(|t| t.shell.id())?;
    prim.set_relationship("material:binding", Some(&id))?;
    Ok(())
}

pub fn bind_ring(prim: &Prim) -> anyhow::Result<()> {
    let id = with_templates(|t| t.ring.id())?;
    prim.set_relationship("material:binding", Some(&id))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `hsd::attributes::material_graph::MAX_NODES`, restated because a guest
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
    /// mismatch shows up as the rejection message `template` prints.
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
