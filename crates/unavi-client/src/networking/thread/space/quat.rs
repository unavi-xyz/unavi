//! Quaternion quantization using smallest-three encoding.

#![expect(clippy::cast_possible_truncation)]
#![expect(clippy::cast_sign_loss)]

use avian3d::math::FRAC_1_SQRT_2;
use bevy::math::Quat;
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

/// Smallest-three encoded quaternion packed into 32 bits.
/// Format: 2-bit index (largest component) + 3×10-bit components.
#[derive(Clone, Copy, Debug, MaxSize, Serialize, Deserialize)]
pub struct PackedQuat(pub u32);

impl From<Quat> for PackedQuat {
    fn from(quat: Quat) -> Self {
        let components = [quat.x, quat.y, quat.z, quat.w];

        // Find largest component by absolute value.
        let (largest_idx, &largest_val) = components
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.abs().partial_cmp(&b.abs()).expect("never fails"))
            .expect("never fails");

        // Ensure largest component is positive (flip entire quaternion if needed).
        let sign = largest_val.signum();

        // Get the three smaller components.
        let mut smaller = [0.0f32; 3];
        let mut j = 0;
        for (i, &c) in components.iter().enumerate() {
            if i != largest_idx {
                smaller[j] = c * sign;
                j += 1;
            }
        }

        // Quantize to 10 bits each. Range [-0.707, 0.707] -> [0, 1023].
        let quantize = |v: f32| -> u32 {
            let normalized = (v / FRAC_1_SQRT_2 + 1.0) * 0.5;
            (normalized.clamp(0.0, 1.0) * 1023.0) as u32
        };

        let a = quantize(smaller[0]);
        let b = quantize(smaller[1]);
        let c = quantize(smaller[2]);

        // Pack: [idx:2][a:10][b:10][c:10].
        let packed = ((largest_idx as u32) << 30) | (a << 20) | (b << 10) | c;

        Self(packed)
    }
}

impl From<PackedQuat> for Quat {
    fn from(packed: PackedQuat) -> Self {
        let packed = packed.0;

        let largest_idx = (packed >> 30) as usize;
        let a = ((packed >> 20) & 0x3FF) as f32;
        let b = ((packed >> 10) & 0x3FF) as f32;
        let c = (packed & 0x3FF) as f32;

        // Dequantize from [0, 1023] -> [-0.707, 0.707].
        let dequantize = |v: f32| -> f32 { (v / 1023.0).mul_add(2.0, -1.0) * FRAC_1_SQRT_2 };

        let smaller = [dequantize(a), dequantize(b), dequantize(c)];

        // Reconstruct largest component.
        let sum_sq: f32 = smaller.iter().map(|x| x * x).sum();
        let largest = (1.0 - sum_sq).max(0.0).sqrt();

        // Rebuild quaternion.
        let mut components = [0.0f32; 4];
        let mut j = 0;
        for (i, c) in components.iter_mut().enumerate() {
            if i == largest_idx {
                *c = largest;
            } else {
                *c = smaller[j];
                j += 1;
            }
        }

        Self::from_xyzw(components[0], components[1], components[2], components[3]).normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::EulerRot;
    use std::f32::consts::PI;

    #[test]
    fn test_packed_quat_identity() {
        let original = Quat::IDENTITY;
        let packed: PackedQuat = original.into();
        let restored: Quat = packed.into();

        let angle = original.angle_between(restored);
        assert!(angle < 0.01, "identity angle error: {angle}");
    }

    #[test]
    fn test_packed_quat_rotations() {
        let test_cases = [
            Quat::from_rotation_x(PI / 4.0),
            Quat::from_rotation_y(PI / 3.0),
            Quat::from_rotation_z(PI / 6.0),
            Quat::from_rotation_x(PI / 2.0),
            Quat::from_euler(EulerRot::XYZ, 0.5, 1.2, -0.3),
        ];

        for original in test_cases {
            let packed: PackedQuat = original.into();
            let restored: Quat = packed.into();

            let angle = original.angle_between(restored);
            assert!(angle < 0.02, "rotation angle error: {angle}");
        }
    }

    #[test]
    fn test_packed_quat_negative_w() {
        // Test quaternion with negative largest component.
        let original = Quat::from_xyzw(0.1, 0.2, 0.3, -0.927).normalize();
        let packed: PackedQuat = original.into();
        let restored: Quat = packed.into();

        let angle = original.angle_between(restored);
        assert!(angle < 0.02, "negative w angle error: {angle}");
    }
}
