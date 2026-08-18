use bevy::prelude::*;
use hsd::id::DocId;
use iroh_docs::NamespaceId;

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct Space(pub NamespaceId);

impl Space {
    /// The space's own document, which is the namespace read as a document id.
    #[must_use]
    pub fn doc_id(&self) -> DocId {
        DocId(*self.0.as_bytes())
    }
}
