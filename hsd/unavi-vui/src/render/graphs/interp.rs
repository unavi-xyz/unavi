//! A fragment network run on the CPU, for the nodes the faces use. Checks the
//! host's kind rules on the way, which a guest otherwise only learns of from
//! a refused graph.

use crate::{
    cells::art::BAYER,
    wired::shading::graph::{
        GraphValue,
        Node,
        Port,
        ShaderGraph,
        SurfaceOutput,
    },
};

/// A value the graph computes: its components, and how many it has, which
/// stands in for its kind (1 float, 2 vec2, 3 vec3, 4 colour).
#[derive(Clone, Copy, Debug)]
pub struct Val {
    pub n: usize,
    pub c: [f32; 4],
}

/// What the host supplies to a fragment.
pub struct Context<'a> {
    pub uv:     [f32; 2],
    pub time:   f32,
    pub scale:  [f32; 3],
    pub screen: [f32; 2],
    /// The fragment's pixel position.
    pub dots:   [f32; 2],
    pub scene:  [f32; 4],
    /// The texel of the image bound at a slot, at a uv.
    pub texel:  &'a dyn Fn(u8, [f32; 2]) -> [f32; 4],
}

fn no_texture(_: u8, _: [f32; 2]) -> [f32; 4] {
    [0.0; 4]
}

impl Context<'static> {
    /// A fragment at `uv`, at time zero, on a unit-sized object over black,
    /// with no texture bound.
    pub const fn at(uv: [f32; 2]) -> Self {
        Self {
            uv,
            time: 0.0,
            scale: [1.0; 3],
            screen: [0.5; 2],
            dots: [0.0; 2],
            scene: [0.0; 4],
            texel: &no_texture,
        }
    }
}

/// What a fragment ends as.
pub struct Shaded {
    pub color: Val,
    clip:      Option<Val>,
}

impl Shaded {
    /// Whether the terminal's threshold cuts the fragment: its alpha is under
    /// the threshold.
    pub fn discarded(&self) -> bool {
        self.clip
            .is_some_and(|threshold| self.color.c[3] < threshold.c[0])
    }
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

fn float(x: f32) -> Val {
    Val {
        n: 1,
        c: [x, 0.0, 0.0, 0.0],
    }
}

fn map(a: Val, f: impl Fn(f32) -> f32) -> Val {
    Val {
        n: a.n,
        c: a.c.map(&f),
    }
}

/// `+ - * /` pair two of one kind or broadcast a float; anything else is a
/// graph the host would refuse.
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

fn length(v: Val) -> Val {
    assert!(v.n > 1, "length of a scalar");
    float(v.c.iter().take(v.n).map(|x| x * x).sum::<f32>().sqrt())
}

fn smoothstep(low: Val, high: Val, x: Val) -> Val {
    let first = matching(low, high, |l, h| h - l);
    let t = matching(first, matching(x, low, |x, l| x - l), |w, d| {
        (d / w).clamp(0.0, 1.0)
    });
    map(t, |t| t * t * 2.0f32.mul_add(-t, 3.0))
}

fn lerp(a: Val, b: Val, t: Val) -> Val {
    assert_eq!(t.n, 1, "lerp factor must be a float");
    assert_eq!(a.n, b.n);
    arithmetic(a, arithmetic(b, a, |b, a| b - a), |a, d| {
        d.mul_add(t.c[0], a)
    })
}

/// The threshold at a pixel, from the same matrix the cell art dithers with.
fn bayer_threshold(x: f32, y: f32) -> f32 {
    let (column, row) = (x.floor() as i32 & 3, y.floor() as i32 & 3);
    let place = usize::try_from(row * 4 + column).unwrap_or(0);
    (f32::from(BAYER[place]) + 0.5) / 16.0
}

/// Runs the fragment network the way the host would, for the subset of nodes
/// the faces use.
pub fn shade(graph: &ShaderGraph, inputs: &[GraphValue], context: &Context) -> Shaded {
    let mut vals: Vec<Val> = Vec::new();
    for node in &graph.surface.nodes {
        let at = |port: Port| gather(&vals, port, inputs);
        let out = match node {
            Node::Uv => Val {
                n: 2,
                c: [context.uv[0], context.uv[1], 0.0, 0.0],
            },
            Node::ScreenUv => Val {
                n: 2,
                c: [context.screen[0], context.screen[1], 0.0, 0.0],
            },
            Node::ScreenDots => Val {
                n: 2,
                c: [context.dots[0], context.dots[1], 0.0, 0.0],
            },
            Node::Dither(op) => {
                let (coord, level) = (at(op.coord), at(op.level));
                assert!(coord.n == 2 && level.n == 1, "dither takes a vec2 and a float");
                float(f32::from(level.c[0] > bayer_threshold(coord.c[0], coord.c[1])))
            }
            Node::Time => float(context.time),
            Node::ObjectScale => Val {
                n: 3,
                c: [context.scale[0], context.scale[1], context.scale[2], 0.0],
            },
            Node::Add(op) => arithmetic(at(op.a), at(op.b), |x, y| x + y),
            Node::Sub(op) => arithmetic(at(op.a), at(op.b), |x, y| x - y),
            Node::Mul(op) => arithmetic(at(op.a), at(op.b), |x, y| x * y),
            Node::Div(op) => arithmetic(at(op.a), at(op.b), |x, y| x / y),
            Node::Min(op) => matching(at(op.a), at(op.b), f32::min),
            Node::Max(op) => matching(at(op.a), at(op.b), f32::max),
            Node::Floor(p) => map(at(*p), f32::floor),
            Node::Fract(p) => map(at(*p), |x| x - x.floor()),
            Node::Abs(p) => map(at(*p), f32::abs),
            Node::Sin(p) => map(at(*p), f32::sin),
            Node::OneMinus(p) => map(at(*p), |x| 1.0 - x),
            Node::Saturate(p) => map(at(*p), |x| x.clamp(0.0, 1.0)),
            Node::Length(p) => length(at(*p)),
            Node::Distance(op) => length(matching(at(op.a), at(op.b), |x, y| x - y)),
            Node::Dot(op) => {
                let v = matching(at(op.a), at(op.b), |x, y| x * y);
                float(v.c.iter().take(v.n).sum())
            }
            Node::Step(op) => matching(at(op.edge), at(op.x), |e, x| f32::from(x >= e)),
            Node::Smoothstep(op) => smoothstep(at(op.low), at(op.high), at(op.x)),
            Node::Lerp(op) => lerp(at(op.a), at(op.b), at(op.t)),
            Node::Select(op) => {
                let (cond, a, b) = (at(op.cond), at(op.a), at(op.b));
                assert_eq!(cond.n, 1, "select condition must be a float");
                assert_eq!(a.n, b.n);
                if cond.c[0] > 0.5 { a } else { b }
            }
            Node::Extract(op) => {
                let v = at(op.v);
                assert!(v.n > 1 && usize::from(op.channel) < v.n);
                float(v.c[usize::from(op.channel)])
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
            Node::TextureSample(op) => {
                let uv = at(op.uv);
                assert_eq!(uv.n, 2, "a texture is sampled at a vec2");
                Val {
                    n: 4,
                    c: (context.texel)(op.slot, [uv.c[0], uv.c[1]]),
                }
            }
            Node::SceneColor(uv) => {
                assert_eq!(at(*uv).n, 2, "the scene is sampled at a vec2");
                Val {
                    n: 4,
                    c: context.scene,
                }
            }
            other => panic!("the interpreter does not run {other:?}"),
        };
        vals.push(out);
    }
    let SurfaceOutput::Unlit(unlit) = &graph.surface.output else {
        panic!("the faces are unlit");
    };
    Shaded {
        color: gather(&vals, unlit.color, inputs),
        clip:  unlit
            .alpha_clip_threshold
            .map(|port| gather(&vals, port, inputs)),
    }
}
