use serde::{
    Deserialize,
    Serialize,
};

pub const VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocMeta {
    pub version: u16,
}

impl Default for DocMeta {
    fn default() -> Self {
        Self { version: VERSION }
    }
}

impl DocMeta {
    pub fn encode(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_stdvec(self)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(bytes)
    }
}
