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
        Combine4Op,
        CullMode,
        Document,
        ExtractOp,
        GraphValue,
        LerpOp,
        Node,
        Port,
        Prim,
        ShaderGraph,
        SurfaceGraph,
        SurfaceOutput,
        UnlitOutput,
        Xform,
    },
};

/// Tint, with the body's alpha in its own alpha channel.
pub const SHELL_TINT: u16 = 0;
pub const SHELL_EMISSIVE: u16 = 1;
/// How far the mote has come toward the attention it is under.
pub const SHELL_HEAT: u16 = 2;
/// Radians of offset into the idle breath, so a form does not pulse as one
/// body.
pub const SHELL_PHASE: u16 = 3;

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

/// A mote's shell: a thin bubble whose edge carries it and whose face lets its
/// contents show through.
///
/// Front faces only. A convex shell drawn front-face-only needs no sorting
/// against itself, and the view term folds every back-facing normal to the
/// same value — so the far wall would be a flat wash rather than a second rim.
fn shell() -> ShaderGraph {
    let mut net = Net::default();
    let tint = Port::Input(SHELL_TINT);
    let heat = Port::Input(SHELL_HEAT);

    // The rim narrows as heat rises, so an attended mote does not merely
    // brighten — its edge thickens, which still reads at a distance where a
    // brightness change does not.
    let power = net.push(Node::Lerp(LerpOp {
        a: cf(3.0),
        b: cf(1.2),
        t: heat,
    }));
    let rim = net.push(Node::Fresnel(power));
    let rim = net.push(Node::Saturate(rim));

    let wave = net.breath(Port::Input(SHELL_PHASE), 1.6);
    let swing = net.push(Node::Mul(binary(wave, cf(0.07))));
    let breath = net.push(Node::Add(binary(swing, cf(1.0))));

    let edge = net.push(Node::Mul(binary(rim, cf(2.4))));
    let lit = net.push(Node::Add(binary(edge, cf(0.35))));
    let lit = net.push(Node::Mul(binary(lit, breath)));
    let brightness = net.push(Node::Mul(binary(Port::Input(SHELL_EMISSIVE), lit)));
    // Past 1.0 on purpose, so the camera's bloom catches an attended mote.
    let rgb = net.push(Node::Mul(binary(tint, brightness)));

    // The face is nearly clear and the edge nearly solid, so the silhouette
    // carries the mote and its contents show through the middle.
    let base = net.push(Node::Extract(ExtractOp {
        v:       tint,
        channel: 3,
    }));
    let alpha = net.push(Node::Lerp(LerpOp {
        a: base,
        b: cf(0.92),
        t: rim,
    }));
    let color = with_alpha(&mut net, rgb, alpha);

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
        ],
        surface:       SurfaceGraph {
            nodes:        net.nodes,
            output:       SurfaceOutput::Unlit(UnlitOutput {
                color,
                alpha_clip_threshold: None,
            }),
            blend:        BlendMode::Blend,
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
    let angle = net.push(Node::Extract(ExtractOp { v: uv, channel: 0 }));
    let across = net.push(Node::Extract(ExtractOp { v: uv, channel: 1 }));

    let filled = net.push(Node::Step(crate::wired::scene::types::StepOp {
        edge: angle,
        x:    progress,
    }));

    // A short bright tail behind the head, which is what makes a fill read as
    // travelling rather than as a bar growing.
    let behind = net.push(Node::Sub(binary(progress, angle)));
    let behind = net.push(Node::Saturate(behind));
    let behind = net.push(Node::OneMinus(behind));
    let head = net.push(Node::Pow(crate::wired::scene::types::PowOp {
        x: behind,
        y: cf(24.0),
    }));
    let head = net.push(Node::Mul(binary(head, filled)));

    // The unfilled arc stays faintly drawn: a cast shows how far it has to go,
    // not only how far it has come.
    let body = net.push(Node::Mul(binary(filled, cf(0.85))));
    let body = net.push(Node::Add(binary(body, cf(0.15))));
    let total = net.push(Node::Add(binary(body, head)));

    // Softened at both edges of the band, so the ring is a ring and not a
    // washer.
    let inner = net.push(Node::Smoothstep(crate::wired::scene::types::SmoothstepOp {
        low:  cf(0.0),
        high: cf(0.35),
        x:    across,
    }));
    let flipped = net.push(Node::OneMinus(across));
    let outer = net.push(Node::Smoothstep(crate::wired::scene::types::SmoothstepOp {
        low:  cf(0.0),
        high: cf(0.35),
        x:    flipped,
    }));
    let band = net.push(Node::Mul(binary(inner, outer)));
    let total = net.push(Node::Mul(binary(total, band)));
    let total = net.push(Node::Mul(binary(total, cf(3.0))));

    let rgb = net.push(Node::Mul(binary(Port::Input(RING_TINT), total)));
    // Alpha pinned: an additive blend scales rgb by alpha, so carrying
    // brightness into alpha as well would square it.
    let color = with_alpha(&mut net, rgb, cf(1.0));

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

fn template(doc: &Document, graph: &ShaderGraph) -> anyhow::Result<Prim> {
    let prim = doc.create_prim()?;
    // Carries a graph and nothing else: with no mesh there is nothing to draw,
    // and the hidden xform keeps it from being mistaken for content.
    prim.set_xform(Some(Xform {
        translation: Vec3::ZERO,
        rotation:    Quat::IDENTITY,
        scale:       Vec3::ZERO,
    }))?;
    prim.set_material_graph(Some(graph))?;
    Ok(prim)
}

fn with_templates<T>(f: impl FnOnce(&Templates) -> T) -> anyhow::Result<T> {
    TEMPLATES.with(|cell| {
        let mut slot = cell.borrow_mut();
        if slot.is_none() {
            let doc = self_document()?;
            *slot = Some(Templates {
                shell: template(&doc, &shell())?,
                ring:  template(&doc, &ring())?,
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
