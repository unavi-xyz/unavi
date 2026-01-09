//! Position quantization types.

use bevy::math::Vec3;
use serde::{Deserialize, Serialize};

/// I-frame position: full precision f32.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct IFramePos {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for IFramePos {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<IFramePos> for Vec3 {
    fn from(p: IFramePos) -> Self {
        Self::new(p.x, p.y, p.z)
    }
}

/// P-frame position: logarithmic-encoded delta from I-frame baseline.
/// Uses ln(1 + |delta| * K) encoding for sub-mm precision near zero.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PFramePos {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

/// Scale factor for logarithmic encoding.
/// With K=1000, we get ~0.03mm precision near zero and ~10m max range.
const LN_SCALE: f32 = 1000.0;
/// Maximum encoded value (i15 magnitude).
const LN_MAX: f32 = 32767.0;

impl PFramePos {
    /// Encode a delta position using logarithmic compression.
    pub fn from_delta(current: Vec3, baseline: Vec3) -> Self {
        let delta = current - baseline;
        Self {
            x: encode_ln(delta.x),
            y: encode_ln(delta.y),
            z: encode_ln(delta.z),
        }
    }

    /// Decode and apply delta to baseline position.
    pub fn apply_to(self, baseline: Vec3) -> Vec3 {
        Vec3::new(
            baseline.x + decode_ln(self.x),
            baseline.y + decode_ln(self.y),
            baseline.z + decode_ln(self.z),
        )
    }
}

/// Encode a single axis delta using ln(1 + |d| * K), preserving sign.
#[expect(clippy::cast_possible_truncation)]
fn encode_ln(delta: f32) -> i16 {
    let sign = delta.signum();
    let magnitude = delta.abs().mul_add(LN_SCALE, 1.0).ln();
    let scaled = (magnitude / 10.0f32.mul_add(LN_SCALE, 1.0).ln() * LN_MAX).min(LN_MAX);
    (sign * scaled) as i16
}

/// Decode: sign * (exp(|v| / scale) - 1) / K.
fn decode_ln(encoded: i16) -> f32 {
    let sign = f32::from(encoded).signum();
    let magnitude = f32::from(encoded).abs() / LN_MAX * 10.0f32.mul_add(LN_SCALE, 1.0).ln();
    sign * magnitude.exp_m1() / LN_SCALE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iframe_pos_roundtrip() {
        let original = Vec3::new(100.5, -42.25, 0.001);
        let iframe: IFramePos = original.into();
        let restored: Vec3 = iframe.into();
        assert!((original - restored).length() < 0.0001);
    }

    #[test]
    fn test_pframe_pos_small_delta() {
        let baseline = Vec3::new(10.0, 5.0, 3.0);
        let current = Vec3::new(10.001, 5.002, 3.0005);

        let pframe = PFramePos::from_delta(current, baseline);
        let restored = pframe.apply_to(baseline);

        let error = (current - restored).length();
        assert!(error < 0.0001, "small delta error: {error}");
    }

    #[test]
    fn test_pframe_pos_medium_delta() {
        let baseline = Vec3::ZERO;
        let current = Vec3::new(1.0, -0.5, 0.25);

        let pframe = PFramePos::from_delta(current, baseline);
        let restored = pframe.apply_to(baseline);

        let error = (current - restored).length();
        assert!(error < 0.01, "medium delta error: {error}");
    }

    #[test]
    fn test_pframe_pos_large_delta() {
        let baseline = Vec3::ZERO;
        let current = Vec3::new(5.0, -3.0, 8.0);

        let pframe = PFramePos::from_delta(current, baseline);
        let restored = pframe.apply_to(baseline);

        let error = (current - restored).length();
        assert!(error < 0.1, "large delta error: {error}");
    }
}
