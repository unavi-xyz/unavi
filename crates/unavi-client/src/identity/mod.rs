use std::sync::Arc;

use bevy::prelude::*;
use unavi_identity::{
    auth::EndpointAuth,
    identity::{
        self,
        NodeIdentity,
    },
};
use unavi_store::local;

mod load;

/// Where this device keeps the state that must outlive the process: its keys
/// and the id of the root document they authored.
#[derive(Resource, Clone)]
pub struct Storage(pub local::Storage);

#[derive(Resource, Clone, Default)]
pub struct SyncConfig {
    pub allow_loopback: bool,
    pub targets:        Vec<String>,
}

/// This device: the user key and the endpoint key.
#[derive(Resource, Clone)]
pub struct LocalNode(pub Arc<NodeIdentity>);

/// The `wired/auth` wiring for this process's endpoint, installed on the
/// endpoint builder and served once the endpoint binds.
#[derive(Resource, Clone)]
pub struct Auth(pub Arc<EndpointAuth>);

pub struct IdentityPlugin {
    pub in_memory: bool,
    pub sync:      SyncConfig,
}

fn key_storage(in_memory: bool) -> local::Storage {
    if in_memory {
        return local::Storage::Ephemeral;
    }

    cfg_select! {
        target_family = "wasm" => local::Storage::Browser,
        _ => local::Storage::Path(unavi_util::dirs::data_local_dir().to_path_buf()),
    }
}

impl Plugin for IdentityPlugin {
    fn build(&self, app: &mut App) {
        let storage = key_storage(self.in_memory);

        let node = match NodeIdentity::load(&storage) {
            Ok(node) => node,
            Err(err) => {
                error!(?err, "failed to load identity key; using an ephemeral one");
                NodeIdentity::load(&local::Storage::Ephemeral).expect("generate identity")
            }
        };
        info!(did = %node.user().did(), "Running as");

        // Published before anything can dial: answering an identity proof runs
        // on a background task with no path back to this world.
        identity::set_local(Arc::clone(node.user()));

        app.insert_resource(Storage(storage))
            .insert_resource(self.sync.clone())
            .insert_resource(LocalNode(Arc::new(node)))
            .insert_resource(Auth(EndpointAuth::new()))
            .add_observer(load::serve_auth)
            .add_observer(load::load_store);
    }
}
