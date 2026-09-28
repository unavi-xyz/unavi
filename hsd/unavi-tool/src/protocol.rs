use serde::{
    Deserialize,
    Serialize,
};
use wired_prelude::{
    wired_math::types::Transform,
    wired_scene::types::Color,
};

pub const CH_DISCOVER: &str = "unavi::tool::discover";
pub const CH_REGISTER: &str = "unavi::tool::register";
pub const CH_ACTIVATE: &str = "unavi::tool::activate";
pub const CH_DEACTIVATE: &str = "unavi::tool::deactivate";
pub const CH_SET_STATE: &str = "unavi::tool::set-state";
pub const CH_TRIGGER: &str = "unavi::tool::trigger";
pub const CH_SCROLL: &str = "unavi::tool::scroll";

#[derive(Serialize, Deserialize)]
pub struct RegisterPayload {
    pub name:         String,
    pub description:  String,
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

#[derive(Serialize, Deserialize)]
pub struct TriggerPayload {
    pub pressed: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ScrollPayload {
    pub delta: f32,
}
