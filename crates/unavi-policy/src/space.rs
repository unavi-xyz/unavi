use bevy::prelude::*;
use hsd::id::DocId;
use iroh_docs::NamespaceId;

use crate::{
    document::DocumentPolicy,
    owner::{
        Owner,
        PolicyOwner,
    },
};

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

/// Entering a space grants its own document the space preset and states the
/// space as its owner. This is where authority is handed to something the
/// local user did not author, and the only place it is.
pub fn grant_space_permissions(
    trigger: On<Add, Space>,
    spaces: Query<&Space>,
    mut commands: Commands,
) {
    let Ok(space) = spaces.get(trigger.entity) else {
        return;
    };
    commands.entity(trigger.entity).insert((
        DocumentPolicy::space(),
        PolicyOwner(Owner::Space(space.doc_id())),
    ));
}
