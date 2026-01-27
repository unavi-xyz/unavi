use std::{fmt, ops::Deref, str::FromStr};

use blake3::Hash;
use loro::LoroValue;
use loro_surgeon::{Hydrate, HydrateError, Reconcile, ReconcileError, loro::LoroMap};
use serde::{Deserialize, Serialize};
use xdid::core::did::Did;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HydratedHash(pub blake3::Hash);

impl Deref for HydratedHash {
    type Target = blake3::Hash;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<blake3::Hash> for HydratedHash {
    fn from(h: blake3::Hash) -> Self {
        Self(h)
    }
}

impl From<HydratedHash> for blake3::Hash {
    fn from(h: HydratedHash) -> Self {
        h.0
    }
}

impl fmt::Display for HydratedHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Hydrate for HydratedHash {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        let LoroValue::Binary(bytes) = value else {
            return Err(HydrateError::TypeMismatch {
                expected: "binary".into(),
                actual: format!("{value:?}").into(),
            });
        };
        let slice: &[u8] = bytes.as_ref();
        let arr: [u8; 32] = slice
            .try_into()
            .map_err(|_| HydrateError::Custom("expected 32 bytes for hash".into()))?;
        Ok(Self(Hash::from_bytes(arr)))
    }
}

impl Reconcile for HydratedHash {
    fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
        Err(ReconcileError::Custom(
            "HydratedHash cannot be reconciled as a root container".into(),
        ))
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        map.insert(key, self.0.as_bytes().to_vec())?;
        Ok(())
    }

    fn to_loro_value(&self) -> Option<LoroValue> {
        Some(LoroValue::Binary(self.0.as_bytes().to_vec().into()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HydratedDid(pub Did);

impl Deref for HydratedDid {
    type Target = Did;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Did> for HydratedDid {
    fn from(d: Did) -> Self {
        Self(d)
    }
}

impl From<HydratedDid> for Did {
    fn from(d: HydratedDid) -> Self {
        d.0
    }
}

impl fmt::Display for HydratedDid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Default for HydratedDid {
    fn default() -> Self {
        Self(
            Did::from_str("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK")
                .expect("valid did"),
        )
    }
}

impl Hydrate for HydratedDid {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        let LoroValue::String(s) = value else {
            return Err(HydrateError::TypeMismatch {
                expected: "string".into(),
                actual: format!("{value:?}").into(),
            });
        };
        let did = Did::from_str(s).map_err(|e| HydrateError::Custom(e.to_string().into()))?;
        Ok(Self(did))
    }
}

impl Reconcile for HydratedDid {
    fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
        Err(ReconcileError::Custom(
            "HydratedDid cannot be reconciled as a root container".into(),
        ))
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        map.insert(key, self.0.to_string())?;
        Ok(())
    }

    fn to_loro_value(&self) -> Option<LoroValue> {
        Some(LoroValue::String(self.0.to_string().into()))
    }
}

#[cfg(test)]
mod tests {
    use loro::LoroDoc;

    use super::*;

    #[test]
    fn roundtrip_content_hash() {
        let doc = LoroDoc::new();
        let map = doc.get_map("test");

        let hash = HydratedHash(blake3::hash(b"test data"));
        hash.reconcile_field(&map, "hash").expect("reconcile");

        let value = map.get_deep_value();
        let LoroValue::Map(m) = &value else {
            panic!("expected map");
        };
        let loaded = HydratedHash::hydrate(m.get("hash").expect("missing")).expect("hydrate");
        assert_eq!(hash, loaded);
    }

    #[test]
    fn roundtrip_wired_did() {
        let doc = LoroDoc::new();
        let map = doc.get_map("test");

        let did: Did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
            .parse()
            .expect("valid did");
        let wdid = HydratedDid(did);
        wdid.reconcile_field(&map, "did").expect("reconcile");

        let value = map.get_deep_value();
        let LoroValue::Map(m) = &value else {
            panic!("expected map");
        };
        let loaded = HydratedDid::hydrate(m.get("did").expect("missing")).expect("hydrate");
        assert_eq!(wdid.0.to_string(), loaded.0.to_string());
    }
}
