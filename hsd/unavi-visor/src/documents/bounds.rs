//! Measuring a document's box from its prims.

use wired_guest::math::Transform;

use super::geometry::{
    Aabb,
    upright,
};
use crate::wired::scene::{
    document::Document,
    properties::{
        Collider,
        Property,
        PropertyKey,
        VertexAttribute,
    },
};

/// Prims read per document; the rest are left out of the box.
const MAX_PRIMS: usize = 8192;

/// The box the document's prims fill in its root frame, from mesh positions
/// and collider shapes. Prims placing other documents count only for their own
/// geometry, since a referenced document has a box of its own. Empty when the
/// document has no geometry.
pub fn measure(doc: &Document) -> Aabb {
    let mut aabb = Aabb::EMPTY;
    let mut budget = MAX_PRIMS;
    for root in doc.roots() {
        gather(doc, root, &Transform::IDENTITY, &mut aabb, &mut budget);
    }
    aabb
}

fn gather(
    doc: &Document,
    prim: (u64, u64),
    parent: &Transform,
    aabb: &mut Aabb,
    budget: &mut usize,
) {
    if *budget == 0 {
        return;
    }
    *budget -= 1;
    let read = doc.get_many(&[
        (prim, PropertyKey::Transform),
        (prim, PropertyKey::MeshVertices(VertexAttribute::Position)),
        (prim, PropertyKey::Collider),
        (prim, PropertyKey::ColliderVertices),
    ]);
    let mut read = read.into_iter();
    let local = match read.next().flatten() {
        Some(Property::Transform(transform)) => transform,
        _ => Transform::IDENTITY,
    };
    let pose = *parent * local;

    if let Some(Property::MeshVertices(stream)) = read.next().flatten() {
        aabb.grow_posed(&stream.values, &pose);
    }
    let shape = read.next().flatten();
    let hull = read.next().flatten();
    if let Some(Property::Collider(shape)) = shape {
        let points = match hull {
            Some(Property::ColliderVertices(points)) => points,
            _ => Vec::new(),
        };
        match shape_box(&shape) {
            Some(local_box) => aabb.union(&local_box.posed(&pose)),
            None => aabb.grow_posed(&points, &pose),
        }
    }

    for child in doc.children(prim) {
        gather(doc, child, &pose, aabb, budget);
    }
}

/// The box of a collider shape in its prim's frame. `None` for a shape made of
/// the prim's collider vertices.
fn shape_box(shape: &Collider) -> Option<Aabb> {
    match shape {
        Collider::Cuboid(extents) => Some(Aabb::centred(*extents)),
        Collider::Sphere(radius) => Some(upright(*radius, *radius)),
        Collider::Capsule(capsule) => Some(upright(
            capsule.height.mul_add(0.5, capsule.radius),
            capsule.radius,
        )),
        Collider::Cylinder(cylinder) => Some(upright(cylinder.height * 0.5, cylinder.radius)),
        Collider::ConvexHull | Collider::Trimesh => None,
    }
}
