use bevy::prelude::*;
use hsd::id::DocId;
use iroh::EndpointId;

/// Who holds a document. Only the host states one.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Owner {
    Peer(EndpointId),
    Space(DocId),
    /// The shell and the tools this node ships.
    System,
}
