use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    byte_array::ByteArray,
    did::HydratedDid,
};

#[cfg_attr(
    feature = "loro",
    derive(loro_surgeon::Hydrate, loro_surgeon::Reconcile)
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconRecord {
    pub did:      HydratedDid,
    pub endpoint: ByteArray<32>,
    pub expires:  i64,
    pub space:    ByteArray<32>,
}

#[cfg(feature = "loro")]
impl BeaconRecord {
    pub fn save(&self, doc: &loro::LoroDoc) -> Result<(), loro_surgeon::error::ReconcileError> {
        use loro_surgeon::{
            Reconcile,
            reconcile::RootReconciler,
        };
        let map = doc.get_map("beacon");
        let rec = RootReconciler::new(map);
        self.reconcile(rec)
    }

    pub fn load(doc: &loro::LoroDoc) -> Result<Self, loro_surgeon::error::HydrateError> {
        use loro_surgeon::Hydrate;
        let map = doc.get_map("beacon");
        Self::hydrate_map(&map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::RecordValue;

    /// Mirrors the host's encode path: build a [`RecordValue`] from a
    /// doc-shaped map, postcard-encode it, then postcard-decode + extract
    /// the typed [`BeaconRecord`] the way the script does.
    #[test]
    fn roundtrip_through_record_value() {
        let endpoint = [7u8; 32];
        let space = [9u8; 32];
        let did = "did:key:z6MkExampleDidForRoundtripTesting".to_string();
        let expires = 1_234_567_890_i64;

        let doc_value = RecordValue::Map(vec![(
            "beacon".into(),
            RecordValue::Map(vec![
                ("did".into(), RecordValue::String(did.clone())),
                ("endpoint".into(), RecordValue::Binary(endpoint.to_vec())),
                ("expires".into(), RecordValue::I64(expires)),
                ("space".into(), RecordValue::Binary(space.to_vec())),
            ]),
        )]);

        let bytes = postcard::to_stdvec(&doc_value).expect("encode");
        let decoded: RecordValue = postcard::from_bytes(&bytes).expect("decode");

        let beacon: BeaconRecord = decoded
            .get("beacon")
            .expect("beacon container present")
            .clone()
            .into_typed()
            .expect("typed beacon");

        assert_eq!(beacon.did.0, did);
        assert_eq!(beacon.endpoint.as_bytes(), &endpoint);
        assert_eq!(beacon.expires, expires);
        assert_eq!(beacon.space.as_bytes(), &space);
    }
}
