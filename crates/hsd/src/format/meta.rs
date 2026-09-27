use serde::{
    Deserialize,
    Serialize,
};

pub const DOC_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocMeta {
    pub version: u16,
}

impl Default for DocMeta {
    fn default() -> Self {
        Self {
            version: DOC_VERSION,
        }
    }
}
