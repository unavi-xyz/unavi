//! Where a document sits in the scene. Per-peer and not persisted.

use bevy::prelude::*;

use crate::{
    document::{
        Hsd,
        Unplaced,
    },
    hierarchy::descends_from,
};

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

/// Puts a document into the scene at `anchor`, or moves one already in it.
pub fn place(doc: &mut EntityWorldMut, anchor: DocAnchor) {
    doc.remove::<Unplaced>().insert(anchor);
}

pub(crate) fn apply_anchors(
    changed: Query<(Entity, &DocAnchor), (With<Hsd>, Without<Unplaced>, Changed<DocAnchor>)>,
    parents: Query<&ChildOf>,
    mut transforms: Query<&mut Transform>,
    mut commands: Commands,
) {
    for (doc_ent, anchor) in &changed {
        match anchor.target {
            // A target under this document would form a `ChildOf` cycle.
            Some(target) if descends_from(target, doc_ent, &parents) => {
                warn!(
                    ?doc_ent,
                    ?target,
                    "anchor target stands under its own document"
                );
                continue;
            }
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
