use bevy::prelude::*;

mod load;

#[cfg(not(target_family = "wasm"))] mod key_pair;

/// Keeps identity and the WDS store off disk, letting several clients share a
/// machine without contending for the same store.
#[derive(Resource, Clone, Copy)]
pub struct InMemory(pub bool);

pub struct IdentityPlugin {
    pub in_memory: bool,
}

impl Plugin for IdentityPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InMemory(self.in_memory))
            .add_observer(load::spawn_actors);
    }
}
