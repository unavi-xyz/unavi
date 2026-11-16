//! The ground slab: the token ground colour with a faint lean toward the
//! document's hue, fading into the sky at its edge.

use wired_guest::{
    LOCKED,
    color::document_color,
    math::{
        Color,
        Transform,
        Vec3,
    },
    style::BG,
};

use crate::{
    horizon,
    unavi::shapes::api::Cuboid,
    wired::scene::{
        document::{
            Document,
            Layer,
        },
        properties::{
            Material,
            Property,
            RigidBody,
        },
    },
    wired::shading::graph::set_graph,
};

pub const GROUND_SIZE: f32 = 30.0;
const GROUND_THICK: f32 = 0.5;

/// How much of the document's hue the ground takes, as a mix fraction of
/// linear colour.
const LEAN: f32 = 0.06;
const METALLIC: f32 = 0.1;
const ROUGHNESS: f32 = 0.85;

/// The ground's colour for the document `id`.
pub fn tint(id: (u64, u64, u64, u64)) -> Color {
    let hue = document_color(id);
    Color::rgb(
        (hue.r - BG.r).mul_add(LEAN, BG.r),
        (hue.g - BG.g).mul_add(LEAN, BG.g),
        (hue.b - BG.b).mul_add(LEAN, BG.b),
    )
}

pub fn build(doc: &Document, id: (u64, u64, u64, u64)) -> anyhow::Result<()> {
    let color = tint(id);
    let shape = Cuboid::new(Vec3::new(GROUND_SIZE, GROUND_THICK, GROUND_SIZE));
    let prim = shape.mesh()?;

    doc.local()
        .set(prim, Property::Collider(shape.collider()))
        .set(prim, Property::RigidBody(RigidBody::static_body()))
        .set(prim, Property::Custom((LOCKED.to_owned(), vec![1])))
        .set(
            prim,
            Property::Transform(Transform::from_translation(Vec3::new(
                0.0,
                -GROUND_THICK / 2.0,
                0.0,
            ))),
        )
        .set(
            prim,
            Property::Material(Material {
                base_color: Some(color),
                metallic: Some(METALLIC),
                roughness: Some(ROUGHNESS),
                ..Material::default()
            }),
        )
        .flush()?;

    set_graph(
        doc,
        prim,
        Layer::Local,
        Some(&horizon::graph(color, METALLIC, ROUGHNESS)),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids() -> impl Iterator<Item = (u64, u64, u64, u64)> {
        (1..200_u64).map(|i| {
            let word = i.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            (word, word.rotate_left(17), word.rotate_left(31), !word)
        })
    }

    #[test]
    fn the_tint_follows_the_id() {
        let id = (1, 2, 3, 4);
        assert_eq!(tint(id), tint(id));
        assert!(ids().any(|other| tint(other) != tint(id)));
    }

    #[test]
    fn the_tint_stays_within_a_faint_lean_of_the_token_ground() {
        for id in ids() {
            let color = tint(id);
            for (got, ground) in [(color.r, BG.r), (color.g, BG.g), (color.b, BG.b)] {
                assert!((got - ground).abs() <= LEAN, "{id:?} leans {got} from {ground}");
            }
        }
    }
}
