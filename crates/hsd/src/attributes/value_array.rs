//! Atomic `Vec<f32>` reconcile/hydrate as a single `LoroValue::List` value.
//!
//! Stores fixed-length float arrays (rotation, translation, scale) as one
//! `LoroValue` rather than a `LoroList` container — so writes are a single
//! `map.insert` op, not N insert/delete ops on a list. Without this, every
//! diff subscriber sees the array mid-rewrite at every intermediate length.

use loro::{LoroMap, LoroValue, ValueOrContainer};
use loro_surgeon::{error::{HydrateError, ReconcileError}, reconcile::MapReconciler};
fn hydrate_with_default(
    map: &LoroMap,
    key: &str,
    default: &[f32],
) -> Result<Vec<f32>, HydrateError> {
    match map.get(key) {
        Some(ValueOrContainer::Value(LoroValue::List(items))) => items
            .iter()
            .map(|v| match v {
                LoroValue::Double(d) => Ok(*d as f32),
                LoroValue::I64(i) => Ok(*i as f32),
                _ => Err(HydrateError::unexpected("number", "other")),
            })
            .collect(),
        Some(ValueOrContainer::Value(LoroValue::Null)) | None => Ok(default.to_vec()),
        Some(_) => Err(HydrateError::unexpected("list value", "other")),
    }
}

fn reconcile_atomic(value: &[f32], m: &mut MapReconciler, key: &str) -> Result<(), ReconcileError> {
    let arr: Vec<LoroValue> = value
        .iter()
        .map(|&f| LoroValue::Double(f64::from(f)))
        .collect();
    let new_value = LoroValue::from(arr);
    if let Some(ValueOrContainer::Value(existing)) = m.map.get(key)
        && existing == new_value
    {
        return Ok(());
    }
    m.map.insert(key, new_value)?;
    Ok(())
}

pub mod rotation {
    use super::{HydrateError, LoroMap, MapReconciler, ReconcileError};

    const DEFAULT: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

    pub fn hydrate(map: &LoroMap, key: &str) -> Result<Vec<f32>, HydrateError> {
        super::hydrate_with_default(map, key, &DEFAULT)
    }

    pub fn reconcile(
        value: &Vec<f32>,
        m: &mut MapReconciler,
        key: &str,
    ) -> Result<(), ReconcileError> {
        super::reconcile_atomic(value, m, key)
    }
}

pub mod scale {
    use super::{HydrateError, LoroMap, MapReconciler, ReconcileError};

    const DEFAULT: [f32; 3] = [1.0, 1.0, 1.0];

    pub fn hydrate(map: &LoroMap, key: &str) -> Result<Vec<f32>, HydrateError> {
        super::hydrate_with_default(map, key, &DEFAULT)
    }

    pub fn reconcile(
        value: &Vec<f32>,
        m: &mut MapReconciler,
        key: &str,
    ) -> Result<(), ReconcileError> {
        super::reconcile_atomic(value, m, key)
    }
}

pub mod translation {
    use super::{HydrateError, LoroMap, MapReconciler, ReconcileError};

    const DEFAULT: [f32; 3] = [0.0, 0.0, 0.0];

    pub fn hydrate(map: &LoroMap, key: &str) -> Result<Vec<f32>, HydrateError> {
        super::hydrate_with_default(map, key, &DEFAULT)
    }

    pub fn reconcile(
        value: &Vec<f32>,
        m: &mut MapReconciler,
        key: &str,
    ) -> Result<(), ReconcileError> {
        super::reconcile_atomic(value, m, key)
    }
}
