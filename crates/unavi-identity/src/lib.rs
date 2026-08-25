use std::sync::Arc;

use bevy::prelude::*;
use wds::identity::{
    RootIdentity,
    store::KeyStorage,
};

mod load;
mod registry;

/// Keeps identity and the WDS store off disk, letting several clients share a
/// machine without contending for the same store.
#[derive(Resource, Clone, Copy)]
pub struct InMemory(pub bool);

/// Which servers this client follows, and whether their `did:web` may resolve
/// to a loopback address.
///
/// A target is named by the operator rather than by a peer, so a loopback
/// address is a local server they chose to run, not an SSRF probe.
#[derive(Resource, Clone, Default)]
pub struct SyncConfig {
    pub allow_loopback: bool,
    pub targets:        Vec<String>,
}

/// The node's identity, loaded before anything that derives a key from it.
#[derive(Resource, Clone)]
pub struct LocalIdentity(pub Arc<RootIdentity>);

pub struct IdentityPlugin {
    pub in_memory: bool,
    pub sync:      SyncConfig,
}

/// Where the client keeps its key.
///
/// An unreadable key is not fatal: the session runs under a generated identity
/// rather than leaving the client with no store at all.
fn key_storage(in_memory: bool) -> KeyStorage {
    if in_memory {
        return KeyStorage::Ephemeral;
    }

    #[cfg(target_family = "wasm")]
    {
        KeyStorage::Browser
    }
    #[cfg(not(target_family = "wasm"))]
    {
        KeyStorage::Path(unavi_util::dirs::data_local_dir().to_path_buf())
    }
}

impl Plugin for IdentityPlugin {
    fn build(&self, app: &mut App) {
        let identity = match RootIdentity::load(&key_storage(self.in_memory)) {
            Ok(identity) => identity,
            Err(err) => {
                error!(?err, "failed to load identity key; using an ephemeral one");
                RootIdentity::load(&KeyStorage::Ephemeral).expect("generate identity")
            }
        };
        info!(did = %identity.did(), "Running as");

        app.insert_resource(InMemory(self.in_memory))
            .insert_resource(self.sync.clone())
            .insert_resource(LocalIdentity(Arc::new(identity)))
            .add_observer(load::spawn_actors);
    }
}
