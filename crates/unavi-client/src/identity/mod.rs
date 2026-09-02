use std::sync::Arc;

use bevy::prelude::*;
use unavi_identity::{
    auth::EndpointAuth,
    identity::NodeIdentity,
    resolve::new_did_resolver,
};
use unavi_space::identity::LocalIdentity;
use unavi_store::local;
use xdid::resolver::DidResolver;

mod load;

/// Where this device keeps the state that must outlive the process: its keys
/// and the id of the root document they authored.
#[derive(Resource, Clone)]
pub struct Storage(pub local::Storage);

#[derive(Resource, Clone, Default)]
pub struct SyncConfig {
    pub targets: Vec<String>,
}

/// This device: the user key and the endpoint key.
#[derive(Resource, Clone)]
pub struct LocalNode(pub Arc<NodeIdentity>);

/// The `wired/auth` wiring for this process's endpoint, installed on the
/// endpoint builder and served once the endpoint binds.
#[derive(Resource, Clone)]
pub struct Auth(pub Arc<EndpointAuth>);

/// Resolves the DIDs this node verifies against. One per process: each carries
/// its own HTTP connection pool.
#[derive(Resource, Clone)]
pub struct Resolve(pub Arc<DidResolver>);

pub struct IdentityPlugin {
    pub storage: local::Storage,
    pub sync:    SyncConfig,
}

/// The storage every client-side plugin shares: nothing on `--in-memory`, the
/// browser on wasm, the app's data directory elsewhere.
pub fn key_storage(in_memory: bool) -> local::Storage {
    if in_memory {
        return local::Storage::Ephemeral;
    }

    cfg_select! {
        target_family = "wasm" => local::Storage::Browser,
        _ => local::Storage::Path(unavi_util::dirs::data_local_dir().to_path_buf()),
    }
}

/// For settings meant to be hand-edited. Ignores `in_memory`, unlike
/// [`key_storage`]: local test instances still want the same keybinds, and
/// the atomic rename in [`Storage::write`](local::Storage::write) makes even
/// simultaneous first-run writes to the one file benign.
///
/// On wasm this is the same `Browser` storage as [`key_storage`], since a
/// browser keeps no separate config bucket.
pub fn config_storage() -> local::Storage {
    cfg_select! {
        target_family = "wasm" => local::Storage::Browser,
        _ => local::Storage::Path(unavi_util::dirs::config_dir().to_path_buf()),
    }
}

impl Plugin for IdentityPlugin {
    fn build(&self, app: &mut App) {
        let storage = self.storage.clone();

        let node = match NodeIdentity::load(&storage) {
            Ok(node) => node,
            Err(err) => {
                error!(?err, "failed to load identity key; using an ephemeral one");
                NodeIdentity::load(&local::Storage::Ephemeral).expect("generate identity")
            }
        };
        info!(did = %node.user().did(), "Running as");

        let resolver = match new_did_resolver() {
            Ok(resolver) => Arc::new(resolver),
            Err(err) => {
                error!(
                    ?err,
                    "failed to build the DID resolver; no peer can be verified"
                );
                return;
            }
        };

        let auth = Arc::new(EndpointAuth::new(
            Arc::clone(node.user()),
            Arc::clone(&resolver),
        ));

        app.insert_resource(LocalIdentity {
            identity: Arc::clone(node.user()),
            bindings: Arc::clone(auth.bindings()),
            resolver: Arc::clone(&resolver),
        })
        .insert_resource(Storage(storage))
        .insert_resource(self.sync.clone())
        .insert_resource(LocalNode(Arc::new(node)))
        .insert_resource(Auth(auth))
        .insert_resource(Resolve(resolver))
        .add_observer(load::serve_auth)
        .add_observer(load::load_store);
    }
}
