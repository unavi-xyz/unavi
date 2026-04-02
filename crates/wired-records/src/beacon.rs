use serde::{Deserialize, Serialize};

use crate::{HydratedDid, HydratedEndpoint, HydratedHash};

#[cfg_attr(
    feature = "loro",
    derive(loro_surgeon::Hydrate, loro_surgeon::Reconcile)
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconRecord {
    pub did: HydratedDid,
    pub endpoint: HydratedEndpoint,
    pub expires: i64,
    pub space: HydratedHash,
}

#[cfg(feature = "loro")]
impl BeaconRecord {
    pub fn load(doc: &loro::LoroDoc) -> anyhow::Result<Self> {
        use loro_surgeon::Hydrate;
        let value = doc.get_map("beacon").get_deep_value();
        Self::hydrate(&value).map_err(|e| anyhow::anyhow!("{e}"))
    }

    pub fn save(&self, doc: &loro::LoroDoc) -> anyhow::Result<()> {
        use loro_surgeon::Reconcile;
        self.reconcile(&doc.get_map("beacon"))?;
        Ok(())
    }
}
