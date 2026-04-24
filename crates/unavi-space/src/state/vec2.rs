use bevy::math::Vec2;
use loro::{LoroMap, LoroValue};
use loro_surgeon::{Hydrate, HydrateError, Reconcile, ReconcileError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HydratedVec2(pub Vec2);

impl Hydrate for HydratedVec2 {
    fn hydrate(value: &loro::LoroValue) -> Result<Self, loro_surgeon::HydrateError> {
        let LoroValue::List(list) = value else {
            return Err(HydrateError::TypeMismatch {
                expected: "list".into(),
                actual: format!("{value:?}").into(),
            });
        };
        let values = list
            .iter()
            .take(2)
            .map(|v| {
                if let LoroValue::Double(f) = v {
                    Ok(*f as f32)
                } else {
                    Err(HydrateError::Custom("expected float".into()))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        if values.len() != 2 {
            return Err(HydrateError::Custom("expected 2 values".into()));
        };
        Ok(Self(Vec2::new(values[0], values[1])))
    }
}

impl Reconcile for HydratedVec2 {
    fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
        Err(ReconcileError::Custom(
            "HydratedVec2 cannot be reconciled as a root container".into(),
        ))
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        let value = LoroValue::List(
            self.0
                .to_array()
                .iter()
                .map(|v| LoroValue::Double(*v as f64))
                .collect(),
        );
        map.insert(key, value)?;
        Ok(())
    }

    fn to_loro_value(&self) -> Option<LoroValue> {
        Some(LoroValue::List(
            self.0
                .to_array()
                .iter()
                .map(|v| LoroValue::Double(*v as f64))
                .collect(),
        ))
    }
}
