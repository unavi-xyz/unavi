use serde::{
    Deserialize,
    Serialize,
};

/// Channel the host emits a [`LinkIntent`] on, postcard-encoded.
pub const INTENT_CHANNEL: &str = "unavi:portal:intent";

/// A portal in `source_space` bearing `link` that no portal in the receiving
/// space answers yet.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LinkIntent {
    pub source_space: [u8; 32],
    pub link:         [u8; 16],
}
