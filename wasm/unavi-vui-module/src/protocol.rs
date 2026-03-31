pub const CH_DISCOVER: &str = "unavi::vui-module::discover";
pub const CH_REGISTER: &str = "unavi::vui-module::register";
pub const CH_ACTIVATE: &str = "unavi::vui-module::activate";
pub const CH_DEACTIVATE: &str = "unavi::vui-module::deactivate";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct RegisterPayload {
    pub name: String,
    pub color: [f32; 4],
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ActivatePayload {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}
