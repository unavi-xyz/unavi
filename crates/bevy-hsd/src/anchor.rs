//! Anchoring: where a document sits, as per-peer runtime state.
//!
//! Deliberately not persisted. Anchoring is per-peer by nature — everyone's
//! gauntlet attaches to *their own* camera — and a document pinned into a
//! space already carries its placement in its root prim's transform.

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

/// Reparents an anchored document's root entity under its target, so placement
/// rides Bevy's ordinary transform propagation rather than a bespoke path.
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
