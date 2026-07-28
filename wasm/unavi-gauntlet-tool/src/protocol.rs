use serde::{
    Deserialize,
    Serialize,
};
use wired_prelude::{
    wired_math::types::Transform,
    wired_scene::types::Color,
};

pub const CH_DISCOVER: &str = "unavi::gauntlet-tool::discover";
pub const CH_REGISTER: &str = "unavi::gauntlet-tool::register";
pub const CH_ACTIVATE: &str = "unavi::gauntlet-tool::activate";
pub const CH_DEACTIVATE: &str = "unavi::gauntlet-tool::deactivate";
pub const CH_SET_STATE: &str = "unavi::gauntlet-tool::set-state";

#[derive(Serialize, Deserialize)]
pub struct RegisterPayload {
    pub name:         String,
    pub icon_prim_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ActivatePayload {
    pub transform: Transform,
}

#[derive(Serialize, Deserialize)]
pub struct ToolStatePayload {
    pub color:  Color,
    pub in_use: bool,
}
