use bevy::prelude::*;
use hsd::id::DocId;
use iroh::EndpointId;

/// Who holds a document.
///
/// Every document resolves to one owner when it is judged, and the class
/// decides how far its content may reach. The host states `System` for the
/// shell and `Space` for a space's own document; a `Peer` is the oldest pin in
/// replicated state. The host is the only one who can state a class, so a peer
/// cannot pin its way into a higher one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Owner {
    Peer(EndpointId),
    Space(DocId),
    /// The node's own content: the shell and the tools it ships.
    System,
}

impl Owner {
    /// Whether content under this owner ignores space membership, as the
    /// shell's does.
    #[must_use]
    pub const fn crosses_space_boundaries(self) -> bool {
        matches!(self, Self::System)
    }
}

/// The host's stated owner for a document, synced into the registry like the
/// rest of its policy.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PolicyOwner(pub Owner);
