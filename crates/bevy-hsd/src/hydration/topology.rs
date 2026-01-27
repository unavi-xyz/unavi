use bevy::mesh::PrimitiveTopology;
use loro::LoroValue;
use loro_surgeon::{Hydrate, HydrateError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HydratedTopology(pub PrimitiveTopology);

impl Hydrate for HydratedTopology {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        let LoroValue::I64(num) = value else {
            return Err(HydrateError::TypeMismatch {
                expected: "i64".into(),
                actual: format!("{value:?}").into(),
            });
        };

        let inner = match num {
            0 => PrimitiveTopology::PointList,
            1 => PrimitiveTopology::LineList,
            2 => PrimitiveTopology::LineStrip,
            3 => PrimitiveTopology::TriangleList,
            4 => PrimitiveTopology::TriangleStrip,
            other => {
                return Err(HydrateError::TypeMismatch {
                    expected: "valid topology".into(),
                    actual: format!("{other}").into(),
                });
            }
        };

        Ok(Self(inner))
    }
}
