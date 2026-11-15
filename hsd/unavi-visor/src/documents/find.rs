//! Finding the documents of the current space by following reference sites.

use std::collections::HashSet;

use unavi_lens::DocumentId;
use wired_guest::LOCK_DEPTH;

use crate::wired::{
    portal::portals::current_space,
    scene::{
        document::{
            Document,
            open_document,
        },
        properties::{
            Property,
            PropertyKey,
        },
    },
};

/// Documents found at once; past this the walk stops.
pub const MAX_DOCUMENTS: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The document the user is standing in.
    Space,
    /// A document some prim of another places by reference.
    Referenced,
}

pub struct Found {
    pub id:   DocumentId,
    pub doc:  Document,
    pub kind: Kind,
}

/// The current space's document and every document reachable from it through
/// prims stating a reference, breadth first, each once. The walk goes
/// [`LOCK_DEPTH`] references deep, as a lock check does, and holds at most
/// [`MAX_DOCUMENTS`]. A document not loaded is passed over. Empty between
/// spaces.
pub fn find() -> Vec<Found> {
    let space = match current_space() {
        Ok(Some(space)) => space,
        Ok(None) => return Vec::new(),
        Err(err) => {
            eprintln!("visor: current space: {err:?}");
            return Vec::new();
        }
    };
    let mut seen = HashSet::from([space]);
    let mut found = Vec::new();
    let mut level = vec![space];
    for depth in 0..=LOCK_DEPTH {
        let mut next = Vec::new();
        for id in level {
            let Some(doc) = open(id) else {
                continue;
            };
            if depth < LOCK_DEPTH {
                for referenced in references(&doc) {
                    if seen.len() < MAX_DOCUMENTS && seen.insert(referenced) {
                        next.push(referenced);
                    }
                }
            }
            found.push(Found {
                id,
                doc,
                kind: if depth == 0 {
                    Kind::Space
                } else {
                    Kind::Referenced
                },
            });
        }
        level = next;
    }
    found
}

fn open(id: DocumentId) -> Option<Document> {
    match open_document(id) {
        Ok(doc) => doc,
        Err(err) => {
            eprintln!("visor: open {id:?}: {err:?}");
            None
        }
    }
}

/// The documents `doc`'s prims place by reference.
fn references(doc: &Document) -> Vec<DocumentId> {
    let keys: Vec<_> = doc
        .prims()
        .into_iter()
        .map(|prim| (prim, PropertyKey::Reference))
        .collect();
    doc.get_many(&keys)
        .into_iter()
        .filter_map(|property| match property {
            Some(Property::Reference(id)) => Some(id),
            _ => None,
        })
        .collect()
}
