use serde::{
    Serialize,
    de::DeserializeOwned,
};
use thiserror::Error;

/// Serialisation on top of the store, plus whatever the host refused, in the
/// terms `wired:error` uses.
#[derive(Debug, Error)]
pub enum TypedKvError {
    #[error("decode failed: {0}")]
    Decode(postcard::Error),
    #[error("encode failed: {0}")]
    Encode(postcard::Error),
    #[error("rate limited; retrying later may succeed")]
    QuotaFlow,
    #[error("out of room; retrying without freeing will not help")]
    QuotaStock,
    #[error("permission denied")]
    Permission,
    #[error("out of reach")]
    Reach,
    #[error("{0}")]
    Other(String),
}

pub trait WiredKv: Sized {
    fn self_kv() -> Self;
    fn get_kv(doc_id: &[u8]) -> Option<Self>;
    fn kv_get(&self, key: &str) -> Option<Vec<u8>>;
    fn kv_set(&self, key: &str, value: &[u8]) -> Result<(), TypedKvError>;
    fn kv_delete(&self, key: &str);
    fn kv_keys(&self) -> Vec<String>;
}

pub struct TypedKv<K> {
    inner: K,
}

impl<K: WiredKv> TypedKv<K> {
    pub const fn new(inner: K) -> Self {
        Self { inner }
    }

    #[must_use]
    pub fn self_kv() -> Self {
        Self::new(K::self_kv())
    }

    #[must_use]
    pub fn get_kv(doc_id: &[u8]) -> Option<Self> {
        K::get_kv(doc_id).map(Self::new)
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, TypedKvError> {
        self.inner.kv_get(key).map_or(Ok(None), |bytes| {
            postcard::from_bytes::<T>(&bytes)
                .map(Some)
                .map_err(TypedKvError::Decode)
        })
    }

    pub fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<(), TypedKvError> {
        let bytes = postcard::to_allocvec(value).map_err(TypedKvError::Encode)?;
        self.inner.kv_set(key, &bytes)
    }

    pub fn delete(&self, key: &str) {
        self.inner.kv_delete(key);
    }

    #[must_use]
    pub fn keys(&self) -> Vec<String> {
        self.inner.kv_keys()
    }
}

impl<K: WiredKv> Default for TypedKv<K> {
    fn default() -> Self {
        Self::self_kv()
    }
}
