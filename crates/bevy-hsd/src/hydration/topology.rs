use bevy::mesh::PrimitiveTopology;
use loro::LoroValue;
use loro_surgeon::{Hydrate, HydrateError, Reconcile, ReconcileError, loro::LoroMap};

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

impl Reconcile for HydratedTopology {
    fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
        Err(ReconcileError::Custom(
            "HydratedTopology cannot be reconciled as a root container".into(),
        ))
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        let num: i64 = match self.0 {
            PrimitiveTopology::PointList => 0,
            PrimitiveTopology::LineList => 1,
            PrimitiveTopology::LineStrip => 2,
            PrimitiveTopology::TriangleList => 3,
            PrimitiveTopology::TriangleStrip => 4,
        };
        map.insert(key, num)?;
        Ok(())
    }

    fn to_loro_value(&self) -> Option<LoroValue> {
        let num: i64 = match self.0 {
            PrimitiveTopology::PointList => 0,
            PrimitiveTopology::LineList => 1,
            PrimitiveTopology::LineStrip => 2,
            PrimitiveTopology::TriangleList => 3,
            PrimitiveTopology::TriangleStrip => 4,
        };
        Some(LoroValue::I64(num))
    }
}
