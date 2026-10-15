//! The frame a script authors in: its own document's root.
//!
//! A prim's `transform` is stated relative to the document that holds it, so
//! every transform a script reads — the camera, a bone, another prim's pose, a
//! ray — is answered in that same frame. The engine world is an implementation
//! detail: loaded spaces sit at different offsets on the floating-origin grid,
//! and none of that difference reaches a script.

use bevy::{
    math::Affine3A,
    prelude::*,
};

/// The affine from a script's document root to the engine world.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame(Affine3A);

impl Frame {
    /// A document whose root is not in this frame's snapshot: reads and
    /// writes are where they are.
    pub const IDENTITY: Self = Self(Affine3A::IDENTITY);

    #[must_use]
    pub fn from_root(root: GlobalTransform) -> Self {
        Self(root.affine())
    }

    /// `point`, in the engine world.
    #[must_use]
    pub fn point_to_world(self, point: Vec3) -> Vec3 {
        self.0.transform_point3(point)
    }

    /// `point`, in this script's frame.
    #[must_use]
    pub fn point_to_frame(self, point: Vec3) -> Vec3 {
        self.0.inverse().transform_point3(point)
    }

    /// `vector` — a direction or a velocity, not a point — in the engine
    /// world. The frame's rotation and scale carry; its translation does not.
    #[must_use]
    pub fn vector_to_world(self, vector: Vec3) -> Vec3 {
        self.0.transform_vector3(vector)
    }

    /// `vector`, in this script's frame.
    #[must_use]
    pub fn vector_to_frame(self, vector: Vec3) -> Vec3 {
        self.0.inverse().transform_vector3(vector)
    }

    /// `transform`, in the engine world.
    #[must_use]
    pub fn transform_to_world(self, transform: Transform) -> Transform {
        Transform::from_matrix((self.0 * transform.compute_affine()).into())
    }

    /// `transform`, in this script's frame.
    #[must_use]
    pub fn transform_to_frame(self, transform: Transform) -> Transform {
        Transform::from_matrix((self.0.inverse() * transform.compute_affine()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A frame that is not the identity, so a missing conversion would show.
    fn tilted() -> GlobalTransform {
        GlobalTransform::from(
            Transform::from_xyz(10.0, -2.0, 3.0)
                .with_rotation(Quat::from_rotation_y(0.7))
                .with_scale(Vec3::splat(2.0)),
        )
    }

    #[test]
    fn a_point_round_trips_through_the_frame() {
        let frame = Frame::from_root(tilted());
        let point = Vec3::new(1.0, 2.0, -3.0);
        let round = frame.point_to_frame(frame.point_to_world(point));
        assert!((round - point).length() < 1.0e-4, "round trip gave {round}");
    }

    #[test]
    fn a_transform_round_trips_through_the_frame() {
        let frame = Frame::from_root(tilted());
        let transform =
            Transform::from_xyz(0.5, -1.0, 2.0).with_rotation(Quat::from_rotation_x(0.3));
        let round = frame.transform_to_frame(frame.transform_to_world(transform));
        assert!(
            (round.translation - transform.translation).length() < 1.0e-4,
            "translation round trip gave {}",
            round.translation
        );
        assert!(
            round.rotation.dot(transform.rotation).abs() > 0.9999,
            "rotation round trip gave {}",
            round.rotation
        );
    }

    #[test]
    fn the_identity_frame_leaves_values_alone() {
        let point = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(Frame::IDENTITY.point_to_frame(point), point);
        assert_eq!(Frame::IDENTITY.point_to_world(point), point);
    }
}
