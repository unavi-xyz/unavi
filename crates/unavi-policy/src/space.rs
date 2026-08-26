use bevy::prelude::*;
use hsd::id::DocId;
use iroh_docs::NamespaceId;

use crate::document::DocumentPolicy;

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

/// Entering a space grants its own document the space tier: the one place
/// authority is handed to something the local user did not author.
pub fn grant_space_permissions(trigger: On<Add, Space>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(DocumentPolicy::space());
}
