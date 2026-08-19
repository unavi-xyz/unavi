//! Anchoring: where a document sits, as per-peer runtime state.
//!
//! Per-peer and not persisted: a document pinned into a space already carries
//! its placement in its root prim's transform.

use bevy::prelude::*;
use unavi_util::hierarchy::descends_from;

use crate::{
    Hsd,
    HsdHeld,
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
///
/// A held document enters whole: the anchor lands with the [`Hsd`] that
/// realizes it, so there is no frame in which it stands anywhere else.
pub fn place(doc: &mut EntityWorldMut, anchor: DocAnchor) {
    match doc.take::<HsdHeld>() {
        Some(held) => doc.insert((anchor, Hsd(held.0))),
        None => doc.insert(anchor),
    };
}

pub fn apply_anchors(
    changed: Query<(Entity, &DocAnchor), (With<Hsd>, Changed<DocAnchor>)>,
    parents: Query<&ChildOf>,
    mut transforms: Query<&mut Transform>,
    mut commands: Commands,
) {
    for (doc_ent, anchor) in &changed {
        match anchor.target {
            // A guest picks the target, and one standing under this document
            // would close the hierarchy into a ring that transform propagation
            // walks forever.
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
