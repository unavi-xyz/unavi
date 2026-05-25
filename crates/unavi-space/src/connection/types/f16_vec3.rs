use half::f16;
use postcard::experimental::max_size::MaxSize;
use serde::{
    Deserialize,
    Serialize,
};

use super::f32_vec3::F32Vec3;

/// Delta position with f16 precision (6 bytes).
/// Range: ±65504, Precision: ~0.1% relative.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
pub struct F16Vec3 {
    pub x: f16,
    pub y: f16,
    pub z: f16,
}

impl MaxSize for F16Vec3 {
    const POSTCARD_MAX_SIZE: usize = 6;
}

impl F16Vec3 {
    pub fn from_delta(current: F32Vec3, baseline: F32Vec3) -> Self {
        Self {
            x: f16::from_f32(current.x - baseline.x),
            y: f16::from_f32(current.y - baseline.y),
            z: f16::from_f32(current.z - baseline.z),
        }
    }

    pub fn apply_to(self, baseline: F32Vec3) -> F32Vec3 {
        F32Vec3 {
            x: baseline.x + self.x.to_f32(),
            y: baseline.y + self.y.to_f32(),
            z: baseline.z + self.z.to_f32(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error(a: F32Vec3, b: F32Vec3) -> f32 {
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        let dz = a.z - b.z;
        dz.mul_add(dz, dy.mul_add(dy, dx * dx)).sqrt()
    }

    #[test]
    fn small_delta() {
        let baseline = F32Vec3 {
            x: 10.0,
            y: 5.0,
            z: 3.0,
        };
        let current = F32Vec3 {
            x: 10.1,
            y: 5.2,
            z: 3.05,
        };

        let pos = F16Vec3::from_delta(current, baseline);
        let restored = pos.apply_to(baseline);

        assert!(error(current, restored) < 0.001);
    }

    #[test]
    fn medium_delta() {
        let baseline = F32Vec3::default();
        let current = F32Vec3 {
            x: 1.0,
            y: -0.5,
            z: 0.25,
        };

        let pos = F16Vec3::from_delta(current, baseline);
        let restored = pos.apply_to(baseline);

        assert!(error(current, restored) < 0.01);
    }

    #[test]
    fn large_delta() {
        let baseline = F32Vec3::default();
        let current = F32Vec3 {
            x: 100.0,
            y: -50.0,
            z: 75.0,
        };

        let pos = F16Vec3::from_delta(current, baseline);
        let restored = pos.apply_to(baseline);

        assert!(error(current, restored) < 0.2);
    }
}
