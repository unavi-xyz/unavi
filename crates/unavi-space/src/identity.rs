//! The local node's identity, as the rest of the app reads it.

use std::sync::Arc;

use bevy::prelude::*;
use iroh_docs::NamespaceId;
use unavi_identity::{
    auth::Bindings,
    identity::Identity,
    resolver::Resolver,
};
use unavi_registry::client::Followed;

/// The node's identity handles, inserted once its keys are loaded.
///
/// Absent until then, so a system that needs an identity tolerates running
/// before one exists rather than assuming the resource.
#[derive(Resource, Clone)]
pub struct LocalIdentity {
    pub identity: Arc<Identity>,
    pub bindings: Arc<Bindings>,
    pub resolver: Arc<Resolver>,
    /// The registries this node follows. Empty until they are reached.
    pub followed: Followed,
}

/// The namespace of this node's root document.
///
/// Inserted once the store has opened it, which is later than
/// [`LocalIdentity`]: the keys load before the store they name a document in.
#[derive(Resource, Clone, Copy)]
pub struct RootDocument(pub NamespaceId);
