use serde::{Deserialize, Serialize};

pub mod f16_vec3;
pub mod f32_vec3;
pub mod i8_vec3;
pub mod pose;
pub mod quat;
pub mod rigid_transform;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct IFrame;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct PFrame;
