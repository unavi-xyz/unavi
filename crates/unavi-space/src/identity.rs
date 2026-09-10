use std::sync::Arc;

use bevy::prelude::*;
use iroh_docs::NamespaceId;
use unavi_identity::{
    auth::bindings::Bindings,
    identity::Identity,
};
use xdid::resolver::DidResolver;

/// The node's identity handles, inserted once its keys are loaded.
///
/// Absent until then, so a system that needs an identity tolerates running
/// before one exists rather than assuming the resource.
#[derive(Resource, Clone)]
pub struct LocalIdentity {
    pub identity: Arc<Identity>,
    pub bindings: Arc<Bindings>,
    pub resolver: Arc<DidResolver>,
}

/// The namespace of this node's root document.
///
/// Inserted once the store has opened it, which is later than
/// [`LocalIdentity`]: the keys load before the store they name a document in.
#[derive(Resource, Clone, Copy)]
pub struct RootDocument(pub NamespaceId);
