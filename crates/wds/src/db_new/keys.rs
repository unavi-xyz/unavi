use native_db::{Key, ToKey};
use serde::{Deserialize, Serialize};
use xdid::core::did::Did;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HashKey(blake3::Hash);

impl HashKey {
    pub fn new(hash: blake3::Hash) -> Self {
        Self(hash)
    }

    pub fn inner(&self) -> &blake3::Hash {
        &self.0
    }
}

impl ToKey for HashKey {
    fn to_key(&self) -> Key {
        Key::new(self.0.as_bytes().to_vec())
    }
    fn key_names() -> Vec<String> {
        vec!["HashKey".to_string()]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DidKey(Did);

impl DidKey {
    pub fn new(did: Did) -> Self {
        Self(did)
    }

    pub fn inner(&self) -> &Did {
        &self.0
    }
}

impl ToKey for DidKey {
    fn to_key(&self) -> Key {
        Key::new(self.0.to_string().into_bytes())
    }
    fn key_names() -> Vec<String> {
        vec!["DidKey".to_string()]
    }
}

/// Wrapper for bool as secondary key.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoolKey(pub bool);

impl ToKey for BoolKey {
    fn to_key(&self) -> Key {
        Key::new(vec![u8::from(self.0)])
    }
    fn key_names() -> Vec<String> {
        vec!["BoolKey".to_string()]
    }
}
