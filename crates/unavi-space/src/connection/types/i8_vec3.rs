use half::f16;
use postcard::experimental::max_size::MaxSize;
use serde::{
    Deserialize,
    Serialize,
};

use super::f16_vec3::F16Vec3;

/// Delta position with 1mm resolution (3 bytes).
/// Range: ±12.7cm from baseline.
#[derive(Clone, Copy, Debug, MaxSize, Serialize, Deserialize, Default)]
pub struct I8Vec3 {
    pub x: i8,
    pub y: i8,
    pub z: i8,
}

impl I8Vec3 {
    pub fn from_delta(current: F16Vec3, baseline: F16Vec3) -> Self {
        let dx = current.x.to_f32() - baseline.x.to_f32();
        let dy = current.y.to_f32() - baseline.y.to_f32();
        let dz = current.z.to_f32() - baseline.z.to_f32();
        Self {
            x: (dx * 1000.0).clamp(-127.0, 127.0) as i8,
            y: (dy * 1000.0).clamp(-127.0, 127.0) as i8,
            z: (dz * 1000.0).clamp(-127.0, 127.0) as i8,
        }
    }

    pub fn apply_to(self, baseline: F16Vec3) -> F16Vec3 {
        F16Vec3 {
            x: f16::from_f32(baseline.x.to_f32() + f32::from(self.x) / 1000.0),
            y: f16::from_f32(baseline.y.to_f32() + f32::from(self.y) / 1000.0),
            z: f16::from_f32(baseline.z.to_f32() + f32::from(self.z) / 1000.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f16v(x: f32, y: f32, z: f32) -> F16Vec3 {
        F16Vec3 {
            x: f16::from_f32(x),
            y: f16::from_f32(y),
            z: f16::from_f32(z),
        }
    }

    fn error(a: F16Vec3, b: F16Vec3) -> f32 {
        let dx = a.x.to_f32() - b.x.to_f32();
        let dy = a.y.to_f32() - b.y.to_f32();
        let dz = a.z.to_f32() - b.z.to_f32();
        dz.mul_add(dz, dy.mul_add(dy, dx * dx)).sqrt()
    }

    /// Walks zero, sub-millimeter, and centimeter-scale deltas, all within the
    /// ±12.7cm clamp range, against the 1mm resolution the format promises.
    #[test]
    fn delta_roundtrip_within_1mm_resolution() {
        let baseline = f16v(1.0, 2.0, 3.0);
        let cases = [
            f16v(1.0, 2.0, 3.0),
            f16v(1.001, 2.002, 3.003),
            f16v(1.05, 1.97, 3.10),
        ];

        for current in cases {
            let pos = I8Vec3::from_delta(current, baseline);
            let restored = pos.apply_to(baseline);
            let err = error(current, restored);
            assert!(err < 0.002, "delta {current:?}: error {err} >= 0.002");
        }
    }

    #[test]
    fn clipping() {
        let baseline = F16Vec3::default();
        let current = f16v(0.2, -0.2, 0.2);

        let pos = I8Vec3::from_delta(current, baseline);
        let restored = pos.apply_to(baseline);

        assert!((restored.x.to_f32() - 0.127).abs() < 0.002);
        assert!((restored.y.to_f32() - (-0.127)).abs() < 0.002);
        assert!((restored.z.to_f32() - 0.127).abs() < 0.002);
    }
}
