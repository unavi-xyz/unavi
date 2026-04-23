use serde::{Deserialize, Serialize};

pub mod f16_vec3;
pub mod f32_vec3;
pub mod i8_vec3;
pub mod pose;
pub mod quat;

#[derive(Serialize, Deserialize, Default)]
pub struct IFrame;

#[derive(Serialize, Deserialize, Default)]
pub struct PFrame;
