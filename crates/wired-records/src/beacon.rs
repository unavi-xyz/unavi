use loro::LoroDoc;
use loro_surgeon::{error::{HydrateError, ReconcileError}, reconcile::RootReconciler, {Hydrate, Reconcile}};
use serde::{Deserialize, Serialize};

use crate::{byte_array::ByteArray, did::HydratedDid};

#[derive(Hydrate, Reconcile, Debug, Clone, Serialize, Deserialize)]
pub struct BeaconRecord {
    pub did: HydratedDid,
    pub endpoint: ByteArray<32>,
    pub expires: i64,
    pub space: ByteArray<32>,
}

impl BeaconRecord {
    pub fn save(&self, doc: &LoroDoc) -> Result<(), ReconcileError> {
        let map = doc.get_map("beacon");
        let rec = RootReconciler::new(map);
        self.reconcile(rec)
    }

    pub fn load(doc: &LoroDoc) -> Result<Self, HydrateError> {
        let map = doc.get_map("beacon");
        Self::hydrate_map(&map)
    }
}
