use std::{
    collections::HashSet,
    sync::{
        Arc,
        LazyLock,
    },
};

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use blake3::Hash;
use loro::{
    LoroDoc,
    LoroMap,
    LoroValue,
    Subscription,
};
use loro_surgeon::{
    Hydrate,
    Reconcile,
    reconcile::RootReconciler,
};
use parking_lot::Mutex;
use serde::{
    Deserialize,
    Serialize,
};
use unavi_util::async_commands::AsyncCommands;

use crate::Space;

pub static SPACE_STATES: LazyLock<Mutex<HashMap<Hash, Arc<SpaceStateRoot>>>> =
    LazyLock::new(Mutex::default);

pub struct SpaceStateRoot {
    pub doc: Arc<LoroDoc>,
    _sub:    Subscription,
}

#[derive(Component)]
pub struct SpaceStateDoc;

#[derive(Hydrate, Reconcile, Default, Debug)]
pub struct SpaceState {
    pub docs: HashSet<Hash>,
}

const ROOT_KEY: &str = "state";
const DOCS_KEY: &str = "docs";

pub fn add_space_state(trigger: On<Add, Space>, spaces: Query<&Space>, mut commands: Commands) {
    let space = spaces.get(trigger.entity).expect("space").0;

    SPACE_STATES.lock().entry(space).or_insert_with(|| {
        let doc = LoroDoc::new();
        let map = doc.get_map(ROOT_KEY);
        let state = SpaceState::default();
        let rec = RootReconciler::new(map);
        state.reconcile(rec).expect("reconcile state");

        let sub = doc.subscribe_local_update(Box::new(move |update| {
            if let Err(err) = AsyncCommands::default()
                .trigger(SpaceStateUpdate {
                    space,
                    data: update.clone(),
                })
                .try_send()
            {
                warn!(?err, "dropped SpaceStateUpdate: async command queue full");
            }
            true
        }));

        Arc::new(SpaceStateRoot {
            doc:  Arc::new(doc),
            _sub: sub,
        })
    });

    commands.entity(trigger.entity).insert(SpaceStateDoc);
}

pub fn remove_space_state(
    trigger: On<Remove, Space>,
    spaces: Query<&Space>,
    mut commands: Commands,
) {
    let space = spaces.get(trigger.entity).expect("space");

    SPACE_STATES.lock().remove(&space.0);

    commands.entity(trigger.entity).remove::<SpaceStateDoc>();
}

#[derive(Event, Clone, Serialize, Deserialize)]
pub struct SpaceStateUpdate {
    pub space: Hash,
    pub data:  Vec<u8>,
}

#[must_use]
pub fn space_state(space: Hash) -> Option<Arc<SpaceStateRoot>> {
    SPACE_STATES.lock().get(&space).cloned()
}

/// Add a document to the public state of a space. No-op if the space is not
/// locally tracked (we don't host it).
pub fn add_doc(space: Hash, doc: Hash) -> bool {
    let Some(root) = space_state(space) else {
        return false;
    };
    let map = root.doc.get_map(ROOT_KEY);
    let docs = match map.get_or_create_container(DOCS_KEY, LoroMap::new()) {
        Ok(m) => m,
        Err(err) => {
            warn!(?err, "failed to access docs map");
            return false;
        }
    };
    if docs.get(&doc.to_string()).is_some() {
        return true;
    }
    if let Err(err) = docs.insert(&doc.to_string(), LoroValue::Null) {
        warn!(?err, "failed to insert doc into space state");
        return false;
    }
    true
}
