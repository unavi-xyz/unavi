use std::{
    collections::BTreeMap,
    sync::{Arc, LazyLock, Mutex},
};

use bevy::{platform::collections::HashMap, prelude::*};
use blake3::Hash;
use loro::{LoroDoc, Subscription};
use loro_surgeon::{Hydrate, Reconcile};
use serde::{Deserialize, Serialize};
use unavi_util::async_commands::try_send_command;
use wired_records::HydratedHash;

use crate::{
    Space,
    peer::{ActiveSpaces, SpaceStateSender},
    state::vec2::HydratedVec2,
};

pub static SPACES: LazyLock<Mutex<HashMap<Hash, SpaceStateRoot>>> = LazyLock::new(Mutex::default);

pub struct SpaceStateRoot {
    pub doc: Arc<LoroDoc>,
    _sub: Subscription,
}

#[derive(Component)]
pub struct SpaceStateDoc(pub Arc<LoroDoc>);

#[derive(Hydrate, Reconcile, Default, Debug)]
pub struct SpaceState {
    portals: BTreeMap<String, PortalState>,
}

#[derive(Hydrate, Reconcile, Debug)]
pub struct PortalState {
    dest_portal: Option<HydratedHash>,
    dest_space: HydratedHash,
    size: HydratedVec2,
}

pub fn add_space_state(trigger: On<Add, Space>, spaces: Query<&Space>, mut commands: Commands) {
    let space = spaces.get(trigger.entity).expect("space").0;

    let doc = Arc::clone(
        &SPACES
            .lock()
            .expect("spaces lock")
            .entry(space)
            .or_insert_with(|| {
                let doc = LoroDoc::new();

                // Init loro doc.
                let map = doc.get_map("state");
                let state = SpaceState::default();
                state.reconcile(&map).expect("reconcile state");

                // Spawn observer to publish local state changes.
                let sub = doc.subscribe_local_update(Box::new(move |update| {
                    let _ = try_send_command(bevy::ecs::system::command::trigger(SpaceStateUpdate {
                        space,
                        data: update.clone(),
                    }));
                    true
                }));

                SpaceStateRoot {
                    doc: Arc::new(doc),
                    _sub: sub,
                }
            })
            .doc,
    );

    commands.entity(trigger.entity).insert(SpaceStateDoc(doc));
}

pub fn remove_space_state(
    trigger: On<Remove, Space>,
    spaces: Query<&Space>,
    mut commands: Commands,
) {
    let space = spaces.get(trigger.entity).expect("space");

    SPACES.lock().expect("spaces lock").remove(&space.0);

    commands.entity(trigger.entity).remove::<SpaceStateDoc>();
}

#[derive(Event, Clone, Serialize, Deserialize)]
pub struct SpaceStateUpdate {
    pub space: Hash,
    pub data: Vec<u8>,
}

pub fn publish_state_update(
    trigger: On<SpaceStateUpdate>,
    peers: Query<(&ActiveSpaces, &SpaceStateSender)>,
) {
    for (spaces, sender) in peers {
        if !spaces.0.contains_key(&trigger.space) {
            continue;
        }
        let _ = sender.0.try_send(trigger.clone());
    }
}
