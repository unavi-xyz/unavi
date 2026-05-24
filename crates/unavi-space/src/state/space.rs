use std::{
    collections::BTreeMap,
    sync::{Arc, LazyLock, Mutex},
};

use bevy::{platform::collections::HashMap, prelude::*};
use blake3::Hash;
use loro::{LoroDoc, Subscription};
use loro_surgeon::{reconcile::RootReconciler, {Hydrate, Reconcile}};
use serde::{Deserialize, Serialize};
use unavi_util::async_commands::AsyncCommands;
use wired_records::byte_array::ByteArray;

use crate::{
    Space,
    peer::{ActiveSpaces, SpaceStateSender},
};

pub static SPACES: LazyLock<Mutex<HashMap<Hash, SpaceStateRoot>>> = LazyLock::new(Mutex::default);

pub struct SpaceStateRoot {
    pub doc: Arc<LoroDoc>,
    _sub: Subscription,
}

#[derive(Component)]
pub struct SpaceStateDoc;

#[derive(Hydrate, Reconcile, Default, Debug)]
pub struct SpaceState {
    portals: BTreeMap<String, PortalState>,
}

#[derive(Hydrate, Reconcile, Debug)]
pub struct PortalState {
    dest_portal: Option<ByteArray<32>>,
    dest_space: ByteArray<32>,
    size_x: f32,
    size_y: f32,
}

pub fn add_space_state(trigger: On<Add, Space>, spaces: Query<&Space>, mut commands: Commands) {
    let space = spaces.get(trigger.entity).expect("space").0;

    SPACES
        .lock()
        .expect("spaces lock")
        .entry(space)
        .or_insert_with(|| {
            let doc = LoroDoc::new();
            let map = doc.get_map("state");
            let state = SpaceState::default();
            let rec = RootReconciler::new(map);
            state.reconcile(rec).expect("reconcile state");

            let sub = doc.subscribe_local_update(Box::new(move |update| {
                let _ = AsyncCommands::default()
                    .trigger(SpaceStateUpdate {
                        space,
                        data: update.clone(),
                    })
                    .try_send();
                true
            }));

            SpaceStateRoot {
                doc: Arc::new(doc),
                _sub: sub,
            }
        });

    commands.entity(trigger.entity).insert(SpaceStateDoc);
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
