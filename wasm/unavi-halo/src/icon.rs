//! What a mote *is*, drawn inside its shell.
//!
//! A slot recognised by silhouette is one that can be found without being
//! read, which a label alone does not give. The surface measures each form
//! and fits it to whatever the mote is drawn at, so a form is authored at any
//! scale — sizing and centering are the shell's business, not the author's.
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
        },
    },
};

/// The unit a form's proportions are authored in. Its absolute size is
/// irrelevant: the surface fits the finished form to the shell.
const R: f32 = 0.42;

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
    Cone { radius: f32, height: f32 },
    Pyramid { radius: f32, height: f32 },
    Cylinder { radius: f32, height: f32 },
}

/// Builds the form its pieces read as.
///
/// Hidden on the way out: a prim nothing has parented is a root of its
/// document and would stand at the origin at full size; the surface places it
/// once it has a mote to sit in. The surface measures the whole tree when it
/// places it, so every piece stands at scale one and the fit rides the root
/// alone.
fn form(pieces: &[Piece], color: Color) -> anyhow::Result<Prim> {
    let root = self_document()?.create_prim()?;
    for piece in pieces {
        let shape = match piece.shape {
            Shape::Cube(size) => Cuboid::new(size).mesh(),
            Shape::Cone { radius, height } => Cone::new(radius, height).mesh(),
            Shape::Pyramid { radius, height } => {
                let cone = Cone::new(radius, height);
                cone.set_resolution(4);
                cone.mesh()
            }
            Shape::Cylinder { radius, height } => Cylinder::new(radius, height).mesh(),
        };
        dress(&shape, color)?;
        shape.set_xform(Some(Transform {
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
/// real beacon is.
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
    // A round cottage: cylinder walls under a pointed cone roof. Simpler than
    // a box and gable, and it reads the same at ring scale.
    let body_radius = R * 0.5;
    let body_height = R * 0.5;
    let roof_radius = R * 0.55;
    let roof_height = R * 0.55;
    // The roof's base floats a hair above the walls, so its flat cap never
    // shares a depth with the wall tops and fights over a pixel.
    let float = R * 0.02;
    vec![
        Piece {
            shape: Shape::Cylinder {
                radius: body_radius,
                height: body_height,
            },
            at:    Vec3::ZERO,
            turn:  Quat::IDENTITY,
        },
        Piece {
            shape: Shape::Cone {
                radius: roof_radius,
                height: roof_height,
            },
            at:    Vec3::new(0.0, body_height * 0.5 + roof_height * 0.5 + float, 0.0),
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
    // A gear shows its face, not its edge: the hub's axis points at the
    // viewer so the teeth read around a circle rather than along a line.
    pieces.iter_mut().for_each(tilt);
    pieces
}

/// Rotates a piece 90° about X, standing the hub's axis out of the surface's
/// plane toward the viewer.
fn tilt(piece: &mut Piece) {
    let s = std::f32::consts::FRAC_1_SQRT_2;
    let turn = Quat::new(s, 0.0, 0.0, s);
    piece.at = turn * piece.at;
    piece.turn = turn * piece.turn;
}

fn tool_pieces() -> Vec<Piece> {
    let radius = R * 0.5;
    let height = R * 1.0;
    vec![
        Piece {
            shape: Shape::Pyramid { radius, height },
            at:    Vec3::new(0.0, height * 0.5, 0.0),
            turn:  Quat::IDENTITY,
        },
        Piece {
            shape: Shape::Pyramid { radius, height },
            at:    Vec3::new(0.0, -height * 0.5, 0.0),
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
    prim.set_xform(Some(Transform {
        translation: Vec3::ZERO,
        rotation:    Quat::IDENTITY,
        scale:       Vec3::ZERO,
    }))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
