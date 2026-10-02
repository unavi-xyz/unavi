//! Wire payloads and channel names shared by the tool and shell sides of
//! `unavi:tool/api`. The WIT is the contract; this is its wire format.

use serde::{
    Deserialize,
    Serialize,
};
use wired_guest::math::Color;

pub const CH_DISCOVER: &str = "unavi:tool/discover";
pub const CH_REGISTER: &str = "unavi:tool/register";
pub const CH_ACTIVATE: &str = "unavi:tool/activate";
pub const CH_DEACTIVATE: &str = "unavi:tool/deactivate";
pub const CH_SET_STATE: &str = "unavi:tool/set-state";
pub const CH_TRIGGER: &str = "unavi:tool/trigger";
pub const CH_SCROLL: &str = "unavi:tool/scroll";

/// Longest `tool` name, matching the bound documented on
/// `unavi:tool/api.tool.constructor`.
pub const MAX_NAME_BYTES: usize = 48;
/// Longest `tool` description, matching the bound documented on
/// `unavi:tool/api.tool.constructor`.
pub const MAX_DESCRIPTION_BYTES: usize = 256;

#[derive(Serialize, Deserialize)]
pub struct RegisterPayload {
    pub name:        String,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
pub struct ToolStatePayload {
    pub color: Color,
}

#[derive(Serialize, Deserialize)]
pub struct TriggerPayload {
    pub pressed: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ScrollPayload {
    pub delta: f32,
}

/// Shortens `s` to at most `max_bytes`, at a char boundary. A `register`
/// payload comes from another document; the `tool` constructor bounds its
/// own name and description, but nothing stops a hand-crafted emitter from
/// sending more.
pub fn truncate(s: &mut String, max_bytes: usize) {
    if s.len() <= max_bytes {
        return;
    }
    let mut end = max_bytes;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s.truncate(end);
}
