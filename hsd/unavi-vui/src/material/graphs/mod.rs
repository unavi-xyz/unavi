//! The shading VUI draws with.
//!
//! `wired:shading` compiles a graph once per distinct value and shares the
//! program across every prim that submits the same one, so a body just
//! submits [`shell`] to itself at creation time rather than being bound to a
//! hidden template prim. What differs between two motes is their overrides.

use wired_guest::math::Color;

use crate::{
    material::graphs::net::{
        Carried,
        Net,
        binary,
        cf,
        with_alpha,
    },
    wired::{
        scene::document::{
            Document,
            Layer,
        },
        shading::graph::{
            BlendMode,
            Combine4Op,
            ConvertOp,
            CullMode,
            ExtractOp,
            GraphValue,
            LerpOp,
            Node,
            Port,
            PowOp,
            ShaderGraph,
            SmoothstepOp,
            SurfaceGraph,
            SurfaceOutput,
            UnlitOutput,
            ValueKind,
            set_graph,
        },
    },
};

#[cfg_attr(
    not(test),
    expect(dead_code, reason = "no surface draws cells from these yet")
)]
mod cell;
#[cfg(test)] mod interp;
pub mod matrix;
#[expect(dead_code, reason = "no surface draws a meter yet")]
pub mod meter;
mod net;
#[expect(dead_code, reason = "no surface draws a picture yet")]
pub mod picture;
pub mod slab;

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
/// front sweeping up the body that dissolves in a dot pattern, so a mote
/// materializes where it stands.
pub const SHELL_BLOOM: u16 = 6;
/// The height of the arrival front's dotted edge, as a share of the body.
const ARRIVAL_FRONT: f32 = 0.4;

pub const RING_TINT: u16 = 0;
pub const RING_PROGRESS: u16 = 1;

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
    let color = net.dissolved(color, Port::Input(SHELL_BLOOM));

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
            GraphValue::Float(crate::tokens::palette::FILM),
            GraphValue::Float(0.0),
            GraphValue::Float(1.0),
        ],
        surface:       SurfaceGraph {
            nodes:        net.nodes,
            output:       SurfaceOutput::Unlit(UnlitOutput {
                color,
                alpha_clip_threshold: Some(cf(0.5)),
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use wired_guest::style;

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
            | Node::ScreenDots
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
            Node::Dither(op) => vec![op.coord, op.level],
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
    fn the_slab_is_well_formed() {
        assert_well_formed(&slab::slab(), "slab");
    }

    #[test]
    fn the_picture_is_well_formed_and_leaves_room_for_the_look() {
        let picture = picture::picture();
        assert_well_formed(&picture, "picture");
        assert_eq!(
            picture
                .surface
                .nodes
                .iter()
                .filter(|node| matches!(node, Node::TextureSample(_)))
                .count(),
            2
        );
    }

    #[test]
    fn the_meter_is_well_formed() {
        assert_well_formed(&meter::meter(), "meter");
    }

    /// Luminance, reading the colour as the linear value the shader works in:
    /// a pessimistic reading, since a gamma encoding would only be darker.
    fn luminance(color: Color) -> f32 {
        0.0722f32.mul_add(color.b, 0.2126f32.mul_add(color.r, 0.7152 * color.g))
    }

    fn contrast(a: f32, b: f32) -> f32 {
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    /// The slab over the worst backdrop there is, black: the room adds
    /// nothing, and the frost's ground and fill are all that is left.
    fn darkest_slab() -> f32 {
        let ground = luminance(style::FROST_GROUND) * style::FROST_GROUND_ALPHA;
        ground.mul_add(
            1.0 - style::FROST_FILL_ALPHA,
            luminance(style::FROST_FILL) * style::FROST_FILL_ALPHA,
        )
    }

    /// Ink reads at 4.5:1 on the darkest slab. The quieter two sit lower,
    /// because the frost's part-white ground over black is itself only a
    /// light grey.
    #[test]
    fn every_ink_reads_on_the_slab_over_any_backdrop() {
        let palette = crate::tokens::palette::Palette::DEFAULT;
        for (name, ink, floor) in [
            ("ink", palette.ink, 4.5),
            ("muted", palette.muted_ink, 2.5),
            ("warning", palette.warning, 2.5),
        ] {
            let ratio = contrast(darkest_slab(), luminance(ink));
            assert!(
                ratio >= floor,
                "{name} ink is {ratio}:1 on the darkest slab, under {floor}:1"
            );
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

    /// How many pixels of one 4×4 tile the arrival keeps at a height of the
    /// body, run through the guest-side interpreter.
    fn kept_in_tile(bloom: f32, up: f32) -> usize {
        let mut net = Net::default();
        let white = Port::Const(GraphValue::Color(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }));
        let color = net.dissolved(white, Port::Input(0));
        let graph = ShaderGraph {
            public_inputs: vec![GraphValue::Float(bloom)],
            surface:       SurfaceGraph {
                nodes:        net.nodes,
                output:       SurfaceOutput::Unlit(UnlitOutput {
                    color,
                    alpha_clip_threshold: Some(cf(0.5)),
                }),
                blend:        BlendMode::Opaque,
                cull:         CullMode::Back,
                cast_shadows: false,
            },
            displacement:  None,
        };

        let mut kept = 0;
        for y in 0..4_u8 {
            for x in 0..4_u8 {
                let context = interp::Context {
                    dots: [f32::from(x), f32::from(y)],
                    ..interp::Context::at([0.5, up])
                };
                if !interp::shade(&graph, &graph.public_inputs, &context).discarded() {
                    kept += 1;
                }
            }
        }
        kept
    }

    /// The front is a dot threshold: whole tiles either side of it, and a
    /// partly kept tile where it passes.
    #[test]
    fn the_arrival_front_dissolves_in_the_bayer_pattern() {
        for up in [0.0, 0.5, 1.0] {
            assert_eq!(kept_in_tile(0.0, up), 0, "nothing has arrived at {up}");
            assert_eq!(kept_in_tile(1.0, up), 16, "all has arrived at {up}");
        }
        assert_eq!(kept_in_tile(0.6, 0.1), 16, "below the front");
        assert_eq!(kept_in_tile(0.6, 0.95), 0, "above the front");
        let on_the_front = kept_in_tile(0.6, 0.7);
        assert!(
            (1..16).contains(&on_the_front),
            "{on_the_front} of 16 kept on the front"
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
            (slab::slab(), "slab"),
            (matrix::matrix(), "matrix"),
            (picture::picture(), "picture"),
            (meter::meter(), "meter"),
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
