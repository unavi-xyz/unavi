use std::sync::Arc;

use bevy::prelude::*;
use wds::identity::{
    WdsIdentity,
    store::KeyStorage,
};

mod load;
mod registry;
mod root_doc;

#[derive(Resource, Clone, Copy)]
pub struct InMemory(pub bool);

#[derive(Resource, Clone, Default)]
pub struct SyncConfig {
    pub allow_loopback: bool,
    pub targets:        Vec<String>,
}

#[derive(Resource, Clone)]
pub struct LocalIdentity(pub Arc<WdsIdentity>);

pub struct IdentityPlugin {
    pub in_memory: bool,
    pub sync:      SyncConfig,
}

fn key_storage(in_memory: bool) -> KeyStorage {
    if in_memory {
        return KeyStorage::Ephemeral;
    }

    cfg_select! {
        target_family = "wasm" => KeyStorage::Browser,
        _ => KeyStorage::Path(unavi_util::dirs::data_local_dir().to_path_buf()),
    }
}

impl Plugin for IdentityPlugin {
    fn build(&self, app: &mut App) {
        let identity = match WdsIdentity::load(&key_storage(self.in_memory)) {
            Ok(identity) => identity,
            Err(err) => {
                error!(?err, "failed to load identity key; using an ephemeral one");
                WdsIdentity::load(&KeyStorage::Ephemeral).expect("generate identity")
            }
        };
        info!(did = %identity.user().did(), "Running as");

        app.insert_resource(InMemory(self.in_memory))
            .insert_resource(self.sync.clone())
            .insert_resource(LocalIdentity(Arc::new(identity)))
            .add_observer(load::spawn_actors);
    }
}
