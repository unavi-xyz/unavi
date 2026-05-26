use serde::{
    Deserialize,
    Serialize,
};
use wired_prelude::{
    wired_math::types::Transform,
    wired_scene::types::Color,
};

pub const CH_DISCOVER: &str = "unavi::vui-module::discover";
pub const CH_REGISTER: &str = "unavi::vui-module::register";
pub const CH_ACTIVATE: &str = "unavi::vui-module::activate";
pub const CH_DEACTIVATE: &str = "unavi::vui-module::deactivate";
pub const CH_SET_COLOR: &str = "unavi::vui-module::set-color";

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
pub struct SetColorPayload {
    pub color: Color,
}
