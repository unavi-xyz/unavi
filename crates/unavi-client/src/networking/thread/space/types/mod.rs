//! Quantized types for efficient pose serialization.

mod f16_pos;
mod f32_pos;
mod i8_pos;
mod pose;
mod quat;

pub use f16_pos::F16Pos;
pub use f32_pos::F32Pos;
pub use i8_pos::I8Pos;
pub use pose::{
    BonePose, IFrameTransform, PFrameRootTransform, PFrameTransform, PlayerIFrame, PlayerPFrame,
};
pub use quat::PackedQuat;
