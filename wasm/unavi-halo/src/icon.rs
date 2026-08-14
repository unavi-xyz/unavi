//! What a mote *is*, drawn inside its shell.
//!
//! A slot recognised by silhouette is one that can be found without being
//! read, which a label alone does not give. Authored to fit a unit sphere,
//! because nothing can measure them — `wired:scene` has no bounds query — so
//! the surface scales them to whatever the mote is drawn at.
//!
//! Every icon is a form rather than a flat glyph: a flat silhouette through a
//! translucent shell reads as nothing at ring scale, while a solid form keeps
//! its shape in any light.

use std::f32::consts::TAU;

use wired_prelude::prelude::*;

use crate::{
    unavi::shapes::api::{
        Cone,
        Cuboid,
        Cylinder,
    },
    wired::scene::{
        api::self_document,
        types::{
            Material,
            Prim,
            Xform,
        },
    },
};

/// The unit a form is authored in, a little under the shell's own radius so
/// the icon reads as sitting inside the shell rather than filling it.
const R: f32 = 0.42;

/// How far a piece sinks into the piece beneath it, so two flat faces never
/// share a depth and fight over a pixel.
const TUCK: f32 = R * 0.05;

/// One piece of a form: a shape posed in its parent's frame.
#[derive(Clone, Copy)]
struct Piece {
    shape: Shape,
    at:    Vec3,
    turn:  Quat,
}

/// A primitive a form is built from.
#[derive(Clone, Copy)]
enum Shape {
    Cube(Vec3),
    Pyramid { radius: f32, height: f32 },
    Cylinder { radius: f32, height: f32 },
}

/// Builds the form its pieces read as.
///
/// Hidden on the way out: a prim nothing has parented is a root of its
/// document and would stand at the origin at full size; the surface places it
/// once it has a mote to sit in. Every piece stands at scale one, so the
/// surface's scale rides the root alone.
fn form(pieces: &[Piece], color: Color) -> anyhow::Result<Prim> {
    let root = self_document()?.create_prim()?;
    for piece in pieces {
        let shape = match piece.shape {
            Shape::Cube(size) => Cuboid::new(size).mesh(),
            Shape::Pyramid { radius, height } => {
                let cone = Cone::new(radius, height);
                cone.set_resolution(4);
                cone.mesh()
            }
            Shape::Cylinder { radius, height } => Cylinder::new(radius, height).mesh(),
        };
        dress(&shape, color)?;
        shape.set_xform(Some(Xform {
            translation: piece.at,
            rotation:    piece.turn,
            scale:       Vec3::ONE,
        }))?;
        root.add_child(&shape)?;
    }
    hide(&root)?;
    Ok(root)
}

/// A real cube for the motes that are one.
pub fn cube(color: Color) -> anyhow::Result<Prim> {
    form(&cube_pieces(), color)
}

/// A house: return, the fixed point.
pub fn home(color: Color) -> anyhow::Result<Prim> {
    form(&home_pieces(), color)
}

/// A cog: the tools.
pub fn tools(color: Color) -> anyhow::Result<Prim> {
    form(&tools_pieces(), color)
}

/// A diamond: one tool among them.
pub fn tool(color: Color) -> anyhow::Result<Prim> {
    form(&tool_pieces(), color)
}

/// The beacon as a form: the cube of corners around a recessed core that the
/// real beacon is, shrunk to the unit-sphere size an icon wears.
pub fn beacon(color: Color) -> anyhow::Result<Prim> {
    form(&beacon_pieces(), color)
}

fn cube_pieces() -> Vec<Piece> {
    vec![Piece {
        shape: Shape::Cube(Vec3::splat(R * 1.35)),
        at:    Vec3::ZERO,
        turn:  Quat::IDENTITY,
    }]
}

fn home_pieces() -> Vec<Piece> {
    let half = R * 0.72;
    let body_height = R * 0.72;
    let roof = R * 0.55;
    let body_at = Vec3::new(0.0, -R * 0.06, 0.0);
    vec![
        Piece {
            shape: Shape::Cube(Vec3::new(half * 2.0, body_height, half * 2.0)),
            at:    body_at,
            turn:  Quat::IDENTITY,
        },
        Piece {
            shape: Shape::Pyramid {
                radius: half,
                height: roof,
            },
            at:    Vec3::new(0.0, body_at.y + body_height * 0.5 + roof * 0.5 - TUCK, 0.0),
            turn:  Quat::IDENTITY,
        },
    ]
}

fn tools_pieces() -> Vec<Piece> {
    let height = R * 0.8;
    let hub = R * 0.5;
    let tooth = R * 0.17;
    let mut pieces = vec![Piece {
        shape: Shape::Cylinder {
            radius: hub,
            height,
        },
        at:    Vec3::ZERO,
        turn:  Quat::IDENTITY,
    }];
    for k in 0..8 {
        let angle = TAU * k as f32 / 8.0;
        pieces.push(Piece {
            shape: Shape::Cube(Vec3::new(tooth * 2.0, height, R * 0.2)),
            at:    Vec3::new(angle.cos(), 0.0, angle.sin()) * (hub + tooth),
            turn:  Quat::new(0.0, (angle * 0.5).sin(), 0.0, (angle * 0.5).cos()),
        });
    }
    pieces
}

fn tool_pieces() -> Vec<Piece> {
    let radius = R * 0.6;
    let height = R * 0.95;
    vec![
        Piece {
            shape: Shape::Pyramid { radius, height },
            at:    Vec3::new(0.0, height * 0.5 - TUCK, 0.0),
            turn:  Quat::IDENTITY,
        },
        Piece {
            shape: Shape::Pyramid { radius, height },
            at:    Vec3::new(0.0, (-height).mul_add(0.5, TUCK), 0.0),
            turn:  Quat::new(1.0, 0.0, 0.0, 0.0),
        },
    ]
}

fn beacon_pieces() -> Vec<Piece> {
    let size = R * 1.7;
    let corner = size * 0.44;
    let offset = size * 0.5 - corner * 0.5;
    let core = size * 0.84;
    let mut pieces = vec![Piece {
        shape: Shape::Cube(Vec3::splat(core)),
        at:    Vec3::ZERO,
        turn:  Quat::IDENTITY,
    }];
    for x in [-1.0_f32, 1.0] {
        for y in [-1.0_f32, 1.0] {
            for z in [-1.0_f32, 1.0] {
                pieces.push(Piece {
                    shape: Shape::Cube(Vec3::splat(corner)),
                    at:    Vec3::new(x, y, z) * offset,
                    turn:  Quat::IDENTITY,
                });
            }
        }
    }
    pieces
}

fn dress(prim: &Prim, color: Color) -> anyhow::Result<()> {
    prim.set_material(Some(Material {
        alpha_cutoff: None,
        alpha_mode:   None,
        base_color:   Some(color),
        // Lit rather than lit-by-the-room: a mote hangs in mid-air, where
        // there is nothing to bounce light off.
        emissive:     Some(Color {
            r: color.r * 0.45,
            g: color.g * 0.45,
            b: color.b * 0.45,
            a: 1.0,
        }),
        double_sided: Some(true),
        metallic:     Some(0.0),
        roughness:    Some(0.6),
    }))?;
    Ok(())
}

fn hide(prim: &Prim) -> anyhow::Result<()> {
    prim.set_xform(Some(Xform {
        translation: Vec3::ZERO,
        rotation:    Quat::IDENTITY,
        scale:       Vec3::ZERO,
    }))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A piece's bounding-box corners, after its own rotation.
    fn corners(piece: &Piece) -> [Vec3; 8] {
        let half = match piece.shape {
            Shape::Cube(size) => size * 0.5,
            Shape::Pyramid { radius, height } => Vec3::new(radius, height * 0.5, radius),
            Shape::Cylinder { radius, height } => Vec3::new(radius, height * 0.5, radius),
        };
        let mut out = [Vec3::ZERO; 8];
        for (i, corner) in out.iter_mut().enumerate() {
            let sign = Vec3::new(
                if i & 1 == 0 { -1.0 } else { 1.0 },
                if i & 2 == 0 { -1.0 } else { 1.0 },
                if i & 4 == 0 { -1.0 } else { 1.0 },
            );
            *corner = piece.at + piece.turn * (half * sign);
        }
        out
    }

    fn forms() -> Vec<(&'static str, Vec<Piece>)> {
        vec![
            ("cube", cube_pieces()),
            ("home", home_pieces()),
            ("tools", tools_pieces()),
            ("tool", tool_pieces()),
            ("beacon", beacon_pieces()),
        ]
    }

    /// Every piece of every form sits inside the unit sphere the surface
    /// scales the icon into: a piece past it would read as a mote bleeding
    /// past its own shell.
    #[test]
    fn every_form_fits_inside_the_unit_sphere() {
        for (name, form) in forms() {
            for piece in &form {
                for corner in corners(piece) {
                    assert!(
                        corner.length() <= 1.0,
                        "{name} pokes past the unit sphere: {corner:?}"
                    );
                }
            }
        }
    }

    /// No form reads as a dot: each reaches past a quarter of the shell's
    /// radius, so a ring of motes separates at a glance.
    #[test]
    fn every_form_reaches_past_a_quarter_of_the_shell() {
        for (name, form) in forms() {
            let reach = form
                .iter()
                .flat_map(|piece| corners(piece))
                .map(|corner| corner.x.abs().max(corner.y.abs()).max(corner.z.abs()))
                .fold(0.0_f32, f32::max);
            assert!(reach >= 0.25, "{name} reads as a dot at {reach}");
        }
    }

    /// The beacon is a core cube with a corner cube on each of its eight
    /// corners; the recessed core is what makes it read as a beacon rather
    /// than as a plain cube.
    #[test]
    fn the_beacon_is_corners_around_a_core() {
        let mut pieces = beacon_pieces();
        assert_eq!(pieces.len(), 9, "a core and eight corners");
        let core = pieces.swap_remove(0);
        assert_eq!(core.at, Vec3::ZERO, "the core sits in the middle");
        let corner = pieces
            .iter()
            .find(|piece| piece.at != Vec3::ZERO)
            .expect("corners");
        let (Shape::Cube(core_size), Shape::Cube(corner_size)) = (core.shape, corner.shape) else {
            panic!("the beacon is built from cubes");
        };
        assert!(
            corner_size.x < core_size.x,
            "the corners poke out of the core"
        );
    }
}
