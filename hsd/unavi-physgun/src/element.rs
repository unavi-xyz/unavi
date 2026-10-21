//! What a pointer hit selects, and the edits made to it whole.
//!
//! | Hit | Element |
//! | --- | --- |
//! | a document a reference site places | the site, in the referencing document |
//! | a prim of the space document | that prim |
//! | a prim of any other document | that document's root prim above it |

use wired_guest::math::Transform;

use crate::{
    element_of,
    unavi::author::api::Edit,
    wired::{
        core::ids::PrimRef,
        scene::{
            document::{
                Document,
                Layer,
                open_document,
            },
            properties::{
                Property,
                PropertyKey,
            },
        },
    },
};

pub type PrimId = (u64, u64);

pub struct Element {
    pub doc:  Document,
    pub prim: PrimId,
    /// A loose document's root, which no prim places, so it cannot be
    /// removed or copied as a prim.
    loose:    bool,
}

impl Element {
    /// The element a hit on `target` selects, or `None` when its document is
    /// not loaded.
    #[must_use]
    pub fn of(target: PrimRef) -> Option<Self> {
        let (doc, prim, loose) = element_of(target)?;
        Some(Self { doc, prim, loose })
    }

    /// Where it is this frame, in the tool's frame.
    pub fn pose(&self) -> anyhow::Result<Transform> {
        self.doc
            .world_transform(self.prim)
            .map_err(|err| anyhow::anyhow!("where the element is: {err:?}"))
    }

    /// Removes it, undoably.
    pub fn delete(&self) -> anyhow::Result<()> {
        if self.loose {
            anyhow::bail!("a loose document cannot be deleted here");
        }
        let edit = Edit::new("Delete")?;
        edit.track_removal(&self.doc, self.prim)?;
        // Every prim of the subtree, deepest first: removing only the root
        // would leave its descendants stated in the document.
        let mut batch = self.doc.local();
        for prim in subtree(&self.doc, self.prim).into_iter().rev() {
            batch = batch.remove(prim);
        }
        batch.flush()?;
        edit.commit()?;
        Ok(())
    }

    /// A copy beside it, built in the local layer and not yet kept. A site's
    /// copy is another site naming the same document, so both show one
    /// element's content.
    pub fn duplicate(&self) -> anyhow::Result<Self> {
        if self.loose {
            anyhow::bail!("a loose document cannot be copied here");
        }
        let parent = match self.doc.get(self.prim, &PropertyKey::Parent) {
            Some(Property::Parent(parent)) => parent,
            _ => None,
        };
        let prim = copy(&self.doc, self.prim, parent)?;
        let doc = open_document(self.doc.id())?
            .ok_or_else(|| anyhow::anyhow!("the document unloaded"))?;
        Ok(Self {
            doc,
            prim,
            loose: false,
        })
    }

    /// Takes a copy back out of the local layer, for one whose edit failed.
    pub fn discard(&self) {
        let mut batch = self.doc.local();
        for prim in subtree(&self.doc, self.prim).into_iter().rev() {
            batch = batch.remove(prim);
        }
        if let Err(err) = batch.flush() {
            eprintln!("build: the failed copy is still here: {err:?}");
        }
    }
}

/// `prim` and every prim beneath it, each after its parent.
fn subtree(doc: &Document, prim: PrimId) -> Vec<PrimId> {
    let mut prims = vec![prim];
    let mut next = 0;
    while let Some(&prim) = prims.get(next) {
        prims.extend(doc.children(prim));
        next += 1;
    }
    prims
}

/// Copies `prim` and everything beneath it under `parent`, answering the
/// copy's root. Relations between the copied prims still name the originals.
fn copy(doc: &Document, prim: PrimId, parent: Option<PrimId>) -> anyhow::Result<PrimId> {
    let copied = doc.create_prim(Layer::Local, parent)?;
    let keys = doc
        .keys(prim)
        .into_iter()
        .filter(|key| !matches!(key, PropertyKey::Parent))
        .map(|key| (prim, key))
        .collect::<Vec<_>>();
    let mut batch = doc.local();
    for property in doc.get_many(&keys).into_iter().flatten() {
        batch = batch.set(copied, property);
    }
    batch.flush()?;
    for child in doc.children(prim) {
        copy(doc, child, Some(copied))?;
    }
    Ok(copied)
}
