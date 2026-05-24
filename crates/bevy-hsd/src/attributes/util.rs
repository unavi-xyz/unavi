use bevy::prelude::*;
use loro::{ContainerID, Index, event::Diff};

use crate::attributes::ParseError;

/// Compose the entity's local `Transform` chain up the `ChildOf` hierarchy
/// without depending on `GlobalTransform` propagation, which only runs in
/// `PostUpdate`.
pub fn compute_global_transform(
    entity: Entity,
    locals: &Query<&Transform>,
    parents: &Query<&ChildOf>,
) -> Transform {
    let mut chain: Vec<Transform> = Vec::new();
    let mut current = Some(entity);
    while let Some(e) = current {
        chain.push(locals.get(e).copied().unwrap_or(Transform::IDENTITY));
        current = parents.get(e).ok().map(ChildOf::parent);
    }
    let mut global = Transform::IDENTITY;
    for local in chain.iter().rev() {
        global = global.mul_transform(*local);
    }
    global
}

/// Parses the top-level updated keys out of a diff map.
pub fn shallow_map_updated_keys(
    path: &[(ContainerID, Index)],
    diff: Diff,
) -> Result<Vec<String>, ParseError> {
    let keys = if path.is_empty() {
        diff.into_map()
            .map_err(|_| anyhow::anyhow!("invalid diff type"))?
            .updated
            .into_keys()
            .map(|s| s.to_string())
            .collect()
    } else {
        vec![
            path[0]
                .1
                .as_key()
                .ok_or_else(|| anyhow::anyhow!("invalid index type"))?
                .to_string(),
        ]
    };
    Ok(keys)
}
