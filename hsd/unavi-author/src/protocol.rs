//! Channel names and payloads between a tool's `edit` and the shell's
//! `history`, both instances of this library on one peer.

use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    snapshot::{
        Snapshot,
        rename_in,
    },
    wired::{
        core::ids::DocumentId,
        event::messaging::{
            self,
            Scope,
        },
        scene::properties::{
            Property,
            PropertyKey,
        },
    },
};

/// Tool to shell: a committed edit, as a postcard [`Filed`].
pub const CH_FILE: &str = "unavi:author/file";
/// Anyone to shell: undo the newest change. Empty payload.
pub const CH_UNDO: &str = "unavi:author/undo";
/// Anyone to shell: redo the newest change undone. Empty payload.
pub const CH_REDO: &str = "unavi:author/redo";

/// Longest edit label.
pub const MAX_LABEL_BYTES: usize = 64;

pub type PrimId = (u64, u64);

/// One part of an edit, as the history replays it in either direction.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Change {
    /// A key went from `before` to `after`. `None` is no value.
    Set {
        doc:    DocumentId,
        prim:   PrimId,
        key:    PropertyKey,
        before: Option<Box<Property>>,
        after:  Option<Box<Property>>,
    },
    /// The snapshot's prims were created.
    Create { doc: DocumentId, prims: Snapshot },
    /// The snapshot's prims were removed.
    Remove { doc: DocumentId, prims: Snapshot },
}

impl Change {
    #[must_use]
    pub const fn doc(&self) -> DocumentId {
        match self {
            Self::Set { doc, .. } | Self::Create { doc, .. } | Self::Remove { doc, .. } => *doc,
        }
    }

    /// Renames prim `old` of `doc` to `new` wherever this change names it,
    /// after a replay built it again under a new id.
    pub fn rename(&mut self, doc: DocumentId, old: PrimId, new: PrimId) {
        if self.doc() != doc {
            return;
        }
        match self {
            Self::Set {
                prim,
                before,
                after,
                ..
            } => {
                if *prim == old {
                    *prim = new;
                }
                for value in [before, after].into_iter().flatten() {
                    rename_in(value, old, new);
                }
            }
            Self::Create { prims, .. } | Self::Remove { prims, .. } => prims.rename(old, new),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Filed {
    pub label:   String,
    pub changes: Vec<Change>,
}

/// Emits an empty request on `channel`, reporting rather than returning a
/// failure: a request nobody hears is the same as one never made.
pub fn request(channel: &str) {
    if let Err(err) = messaging::emit(channel, &[], None, Scope::Global) {
        eprintln!("author: request on {channel}: {err:?}");
    }
}

/// The key a property is stored under.
#[must_use]
pub fn key_of(property: &Property) -> PropertyKey {
    match property {
        Property::Parent(_) => PropertyKey::Parent,
        Property::Name(_) => PropertyKey::Name,
        Property::Transform(_) => PropertyKey::Transform,
        Property::Reference(_) => PropertyKey::Reference,
        Property::MeshTopology(_) => PropertyKey::MeshTopology,
        Property::MeshVertices(stream) => PropertyKey::MeshVertices(stream.attribute),
        Property::MeshIndices(_) => PropertyKey::MeshIndices,
        Property::Material(_) => PropertyKey::Material,
        Property::ImageSampler(_) => PropertyKey::ImageSampler,
        Property::ImageData(_) => PropertyKey::ImageData,
        Property::Text(_) => PropertyKey::Text,
        Property::Collider(_) => PropertyKey::Collider,
        Property::ColliderVertices(_) => PropertyKey::ColliderVertices,
        Property::ColliderIndices(_) => PropertyKey::ColliderIndices,
        Property::RigidBody(_) => PropertyKey::RigidBody,
        Property::GravityScale(_) => PropertyKey::GravityScale,
        Property::Sky(_) => PropertyKey::Sky,
        Property::Portal(_) => PropertyKey::Portal,
        Property::Spawn(_) => PropertyKey::Spawn,
        Property::Relation((relation, _)) => PropertyKey::Relation(relation.clone()),
        Property::Custom((name, _)) => PropertyKey::Custom(name.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: DocumentId = (1, 2, 3, 4);
    const OTHER: DocumentId = (5, 6, 7, 8);
    const OLD: PrimId = (1, 1);
    const NEW: PrimId = (9, 9);

    fn set(doc: DocumentId, prim: PrimId, parent: PrimId) -> Change {
        Change::Set {
            doc,
            prim,
            key: PropertyKey::Parent,
            before: Some(Box::new(Property::Parent(Some(parent)))),
            after: None,
        }
    }

    #[test]
    fn a_rebuilt_prim_is_followed_where_it_is_edited_and_where_it_is_a_parent() {
        let mut edited = set(DOC, OLD, (2, 2));
        edited.rename(DOC, OLD, NEW);
        assert!(matches!(edited, Change::Set { prim: NEW, .. }));

        let mut child = set(DOC, (3, 3), OLD);
        child.rename(DOC, OLD, NEW);
        let Change::Set { before, .. } = child else {
            unreachable!("built as a set");
        };
        assert_eq!(before.as_deref(), Some(&Property::Parent(Some(NEW))));
    }

    #[test]
    fn a_rename_in_one_document_leaves_another_alone() {
        let mut elsewhere = set(OTHER, OLD, OLD);
        elsewhere.rename(DOC, OLD, NEW);
        assert!(matches!(elsewhere, Change::Set { prim: OLD, .. }));
    }
}
