use const_format::concatcp;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    prop_name,
    property::{
        Payload,
        Property,
        name::PropName,
        render_bytes,
    },
};

pub const GROUP: &str = "script";

/// Wasm component bytes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ScriptAttr(pub Vec<u8>);

impl Property for ScriptAttr {
    const NAME: PropName = prop_name!(concatcp!(GROUP, "/wasm"));

    fn render(payload: &[u8]) -> String {
        match Self::decode(payload) {
            Ok(value) => render_bytes(value.0.len()),
            Err(err) => format!("<undecodable: {err}>"),
        }
    }
}
