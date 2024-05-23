use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use dwn::{
    store::{DataStore, MessageStore},
    DWN,
};
use tokio::sync::RwLock;

pub struct GlobalContext<D: DataStore, M: MessageStore> {
    pub dwn: Arc<DWN<D, M>>,
    pub instances: RwLock<HashMap<String, Instance>>,
    pub world_host_did: String,
}

pub struct Instance {
    pub players: BTreeMap<u32, Player>,
}

pub struct Player {
    /// Maps a global server player id -> local id.
    pub local_ids: BTreeMap<u32, u16>,
}
