//! The one wrong thing a home holds, chosen by the document's id. A thing is
//! a list of boxes; adding one is a variant, a module of parts and an arm in
//! [`Thing::parts`].

mod door;
mod part;
mod placement;
mod stairs;

use wired_guest::{
    LOCKED,
    color::document_color,
    math::{
        Quat,
        Transform,
        Vec3,
    },
};

use self::{
    part::Part,
    placement::placement,
};
use crate::{
    unavi::shapes::api::Cuboid,
    wired::scene::{
        document::{
            Document,
            Layer,
        },
        properties::{
            Property,
            RigidBody,
        },
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Thing {
    /// A door in a frame with no wall.
    Door,
    /// Five steps that end in air.
    Stairs,
}

impl Thing {
    const ALL: [Self; 2] = [Self::Door, Self::Stairs];

    fn parts(self) -> Vec<Part> {
        match self {
            Self::Door => door::parts(),
            Self::Stairs => stairs::parts(),
        }
    }
}

pub const fn choose(id: (u64, u64, u64, u64)) -> Thing {
    Thing::ALL[(id.3 % Thing::ALL.len() as u64) as usize]
}

/// Builds the thing `id` picks where `id` places it, locked like the ground.
pub fn build(doc: &Document, id: (u64, u64, u64, u64)) -> anyhow::Result<()> {
    let pose = placement(id);
    let hue = document_color(id);

    let root = doc.create_prim(Layer::Local, None)?;
    let mut batch = doc
        .local()
        .set(
            root,
            Property::Transform(Transform::new(
                pose.position,
                Quat::from_rotation_y(pose.yaw),
                Vec3::ONE,
            )),
        )
        .set(root, Property::Custom((LOCKED.to_owned(), vec![1])));

    for part in choose(id).parts() {
        let shape = Cuboid::new(part.size);
        let prim = shape.mesh()?;
        batch = batch
            .set(prim, Property::Parent(Some(root)))
            .set(prim, Property::Material(part.paint.material(hue)))
            .set(
                prim,
                Property::Transform(Transform::from_translation(part.centre)),
            );
        if part.solid {
            batch = batch
                .set(prim, Property::Collider(shape.collider()))
                .set(prim, Property::RigidBody(RigidBody::static_body()));
        }
    }

    batch.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_choice_follows_the_id() {
        let id = (1, 2, 3, 4);
        assert_eq!(choose(id), choose(id));
        assert_ne!(choose((0, 0, 0, 0)), choose((0, 0, 0, 1)));
    }

    #[test]
    fn every_thing_is_chosen_by_some_id() {
        for thing in Thing::ALL {
            assert!((0..Thing::ALL.len() as u64).any(|i| choose((0, 0, 0, i)) == thing));
        }
    }

    #[test]
    fn every_thing_has_a_collider() {
        for thing in Thing::ALL {
            assert!(thing.parts().iter().any(|p| p.solid), "{thing:?}");
        }
    }
}
