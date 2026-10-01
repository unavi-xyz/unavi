#[cfg(target_family = "wasm")] use std::path::PathBuf;
use std::sync::Arc;
#[cfg(not(target_family = "wasm"))] use std::sync::LazyLock;

use bevy::prelude::*;
#[cfg(not(target_family = "wasm"))]
use directories::ProjectDirs;
use unavi_identity::{
    auth::EndpointAuth,
    identity::NodeIdentity,
    resolver::Resolver,
};
use unavi_registry::client::Followed;
use unavi_space::identity::LocalIdentity;

mod load;

/// Where this device keeps the state that must outlive the process: its keys
/// and the id of the root document they authored.
#[derive(Resource, Clone)]
pub struct KeyStorage(pub unavi_local::DeviceStorage);

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

pub struct IdentityPlugin {
    pub storage: unavi_local::DeviceStorage,
    pub sync:    SyncConfig,
}

/// The app's data and config directories, created on first use.
#[cfg(not(target_family = "wasm"))]
static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-client").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    std::fs::create_dir_all(dirs.config_dir()).expect("config dir");
    dirs
});

/// The storage every client-side plugin shares: nothing on `--in-memory`, the
/// app's data root elsewhere — a local-storage prefix on wasm.
pub fn key_storage(in_memory: bool) -> unavi_local::DeviceStorage {
    if in_memory {
        return unavi_local::DeviceStorage::memory();
    }

    unavi_local::DeviceStorage::at(cfg_select! {
        target_family = "wasm" => PathBuf::from("data"),
        _ => DIRS.data_local_dir().to_path_buf(),
    })
}

/// For settings meant to be hand-edited. Ignores `in_memory`, unlike
/// [`key_storage`]: local test instances still want the same keybinds, and
/// the atomic rename in
/// [`DeviceStorage::write`](unavi_local::DeviceStorage::write) makes even
/// simultaneous first-run writes to the one file benign.
///
/// The root is the config directory on native, the `config` local-storage
/// prefix on wasm — apart from [`key_storage`]'s `data` root, exactly as on
/// native.
pub fn config_storage() -> unavi_local::DeviceStorage {
    unavi_local::DeviceStorage::at(cfg_select! {
        target_family = "wasm" => PathBuf::from("config"),
        _ => DIRS.config_dir().to_path_buf(),
    })
}

impl Plugin for IdentityPlugin {
    fn build(&self, app: &mut App) {
        let storage = self.storage.clone();

        let node = match NodeIdentity::load(&storage) {
            Ok(node) => node,
            Err(err) => {
                error!(?err, "failed to load identity key; using an ephemeral one");
                NodeIdentity::load(&unavi_local::DeviceStorage::memory())
                    .expect("generate identity")
            }
        };
        info!(did = %node.user().did(), "Running as");

        let resolver = match Resolver::new() {
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
            followed: Followed::default(),
        })
        .insert_resource(KeyStorage(storage))
        .insert_resource(self.sync.clone())
        .insert_resource(LocalNode(Arc::new(node)))
        .insert_resource(Auth(auth))
        .add_observer(load::serve_auth)
        .add_observer(load::load_store);
    }
}
