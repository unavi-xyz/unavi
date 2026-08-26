use std::sync::Arc;

use bevy::prelude::*;
use parking_lot::RwLock;
use unavi_identity::{
    auth::bindings::Bindings,
    identity::Identity,
    resolve::Resolver,
};

/// The node's identity handles, inserted once its keys are loaded.
///
/// Absent until then, so a system that needs an identity tolerates running
/// before one exists rather than assuming the resource.
#[derive(Resource, Clone)]
pub struct LocalIdentity {
    pub identity: Arc<Identity>,
    pub bindings: Arc<Bindings>,
    pub resolver: Arc<Resolver>,
}

/// A handle for the connection layer, which runs on spawned tasks with no path
/// to the ECS world.
static INSTALLED: RwLock<Option<LocalIdentity>> = RwLock::new(None);

pub fn install(local: LocalIdentity) {
    *INSTALLED.write() = Some(local);
}

#[must_use]
pub fn local() -> Option<LocalIdentity> {
    INSTALLED.read().clone()
}

#[must_use]
pub fn bindings() -> Option<Arc<Bindings>> {
    INSTALLED.read().as_ref().map(|l| Arc::clone(&l.bindings))
}

/// Runs at startup, once every plugin has had the chance to insert the
/// resource, so the connection layer and policy share one table.
pub fn install_local(local: Option<Res<LocalIdentity>>) {
    let Some(local) = local else {
        return;
    };
    crate::trust::install_resolver(Arc::clone(&local.bindings));
    install(local.clone());
}
