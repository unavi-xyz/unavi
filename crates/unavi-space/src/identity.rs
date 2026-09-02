use std::sync::Arc;

use bevy::prelude::*;
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
