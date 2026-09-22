use serde::{
    Deserialize,
    Serialize,
};

use crate::attributes::Attribute;

/// The wasm component bytes, held in the payload. Presence of the attribute is
/// what attaches the script.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptAttr(pub Vec<u8>);

impl Attribute for ScriptAttr {
    const KEY: &'static str = "script";
}
