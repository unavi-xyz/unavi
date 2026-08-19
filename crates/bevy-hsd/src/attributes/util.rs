use bevy::prelude::*;
use unavi_util::hierarchy::ancestors;

/// Composes the entity's local `Transform` chain up the `ChildOf` hierarchy
/// without depending on `GlobalTransform` propagation, which only runs in
/// `PostUpdate`.
#[must_use]
pub fn compute_global_transform(
    entity: Entity,
    locals: &Query<&Transform>,
    parents: &Query<&ChildOf>,
) -> Transform {
    ancestors(entity, parents).fold(Transform::IDENTITY, |global, at| {
        locals
            .get(at)
            .copied()
            .unwrap_or(Transform::IDENTITY)
            .mul_transform(global)
    })
}
