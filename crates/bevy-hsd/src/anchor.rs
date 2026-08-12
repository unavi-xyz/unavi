//! Anchoring: where a document sits, as per-peer runtime state.
//!
//! Per-peer and not persisted: a document pinned into a space already carries
//! its placement in its root prim's transform.

use bevy::prelude::*;

use crate::Hsd;

#[derive(Component, Debug, Clone, Copy)]
#[require(Transform)]
pub struct DocAnchor {
    /// `None` anchors to the space root.
    pub target: Option<Entity>,
    pub offset: Transform,
}

impl DocAnchor {
    #[must_use]
    pub const fn root(offset: Transform) -> Self {
        Self {
            target: None,
            offset,
        }
    }
}

pub fn apply_anchors(
    changed: Query<(Entity, &DocAnchor), (With<Hsd>, Changed<DocAnchor>)>,
    mut transforms: Query<&mut Transform>,
    mut commands: Commands,
) {
    for (doc_ent, anchor) in &changed {
        match anchor.target {
            Some(target) => {
                commands.entity(doc_ent).insert(ChildOf(target));
            }
            None => {
                commands.entity(doc_ent).remove::<ChildOf>();
            }
        }
        if let Ok(mut transform) = transforms.get_mut(doc_ent) {
            *transform = anchor.offset;
        }
    }
}
