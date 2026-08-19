use bevy::prelude::*;

mod load;

#[cfg(not(target_family = "wasm"))] mod key_pair;

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

pub struct IdentityPlugin {
    pub in_memory: bool,
    pub sync:      SyncConfig,
}

impl Plugin for IdentityPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InMemory(self.in_memory))
            .insert_resource(self.sync.clone())
            .add_observer(load::spawn_actors);
    }
}
