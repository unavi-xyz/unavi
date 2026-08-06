use serde::{
    Deserialize,
    Serialize,
};

/// The kinds a graph node's output (or a public input) can have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueKind {
    Float,
    Vec2,
    Vec3,
    Color,
}

impl ValueKind {
    #[must_use]
    pub const fn components(self) -> u8 {
        match self {
            Self::Float => 1,
            Self::Vec2 => 2,
            Self::Vec3 => 3,
            Self::Color => 4,
        }
    }

    #[must_use]
    pub const fn is_vector(self) -> bool {
        self.components() > 1
    }
}

/// A constant literal a [`Port`](super::node::Port) can carry.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GraphValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Color([f32; 4]),
}

impl GraphValue {
    #[must_use]
    pub const fn kind(&self) -> ValueKind {
        match self {
            Self::Float(_) => ValueKind::Float,
            Self::Vec2(_) => ValueKind::Vec2,
            Self::Vec3(_) => ValueKind::Vec3,
            Self::Color(_) => ValueKind::Color,
        }
    }
}

/// Whether every component is a real number.
///
/// `NaN` and the infinities have no WGSL literal — `f32`'s own formatting
/// renders them `NaN`/`inf`, which no shader compiler accepts — so they are
/// refused here rather than in any one backend.
#[must_use]
pub const fn is_finite(value: GraphValue) -> bool {
    match value {
        GraphValue::Float(v) => v.is_finite(),
        GraphValue::Vec2([x, y]) => x.is_finite() && y.is_finite(),
        GraphValue::Vec3([x, y, z]) => x.is_finite() && y.is_finite() && z.is_finite(),
        GraphValue::Color([r, g, b, a]) => {
            r.is_finite() && g.is_finite() && b.is_finite() && a.is_finite()
        }
    }
}
