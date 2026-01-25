use std::collections::BTreeMap;

use loro::{LoroMap, LoroValue};
use smol_str::SmolStr;

use crate::ReconcileError;

/// Write a Rust value into a Loro container.
pub trait Reconcile {
    /// Reconcile this value into a [`LoroMap`].
    ///
    /// For struct types, this writes all fields to the map.
    ///
    /// # Errors
    ///
    /// Returns [`ReconcileError`] if the Loro operation fails.
    fn reconcile(&self, map: &LoroMap) -> Result<(), ReconcileError>;

    /// Reconcile this value as a field in a [`LoroMap`].
    ///
    /// For primitive types, this inserts the value directly.
    /// For struct types, this creates a nested map and calls `reconcile`.
    ///
    /// # Errors
    ///
    /// Returns [`ReconcileError`] if the Loro operation fails.
    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError>;

    /// Convert to a [`LoroValue`] for use in lists.
    ///
    /// Returns `Some` for primitive-like types that can be pushed directly to lists.
    /// Returns `None` for struct types that need nested maps.
    fn to_loro_value(&self) -> Option<LoroValue> {
        None
    }
}

impl Reconcile for bool {
    fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
        Err(ReconcileError::Custom(
            "bool cannot be reconciled as a root container".into(),
        ))
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        map.insert(key, *self)?;
        Ok(())
    }

    fn to_loro_value(&self) -> Option<LoroValue> {
        Some(LoroValue::Bool(*self))
    }
}

impl Reconcile for i64 {
    fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
        Err(ReconcileError::Custom(
            "i64 cannot be reconciled as a root container".into(),
        ))
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        map.insert(key, *self)?;
        Ok(())
    }

    fn to_loro_value(&self) -> Option<LoroValue> {
        Some(LoroValue::I64(*self))
    }
}

impl Reconcile for f64 {
    fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
        Err(ReconcileError::Custom(
            "f64 cannot be reconciled as a root container".into(),
        ))
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        map.insert(key, *self)?;
        Ok(())
    }

    fn to_loro_value(&self) -> Option<LoroValue> {
        Some(LoroValue::Double(*self))
    }
}

impl Reconcile for String {
    fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
        Err(ReconcileError::Custom(
            "String cannot be reconciled as a root container".into(),
        ))
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        map.insert(key, self.as_str())?;
        Ok(())
    }

    fn to_loro_value(&self) -> Option<LoroValue> {
        Some(LoroValue::String(self.as_str().into()))
    }
}

impl Reconcile for Vec<u8> {
    fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
        Err(ReconcileError::Custom(
            "Vec<u8> cannot be reconciled as a root container".into(),
        ))
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        map.insert(key, self.as_slice())?;
        Ok(())
    }

    fn to_loro_value(&self) -> Option<LoroValue> {
        Some(LoroValue::Binary(self.clone().into()))
    }
}

impl<T: Reconcile> Reconcile for Option<T> {
    fn reconcile(&self, map: &LoroMap) -> Result<(), ReconcileError> {
        self.as_ref()
            .map_or_else(|| Ok(()), |inner| inner.reconcile(map))
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        if let Some(inner) = self {
            inner.reconcile_field(map, key)
        } else {
            map.insert(key, loro::LoroValue::Null)?;
            Ok(())
        }
    }
}

impl<T: Reconcile> Reconcile for Vec<T> {
    fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
        Err(ReconcileError::Custom(
            "Vec<T> cannot be reconciled as a root container".into(),
        ))
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        let list = map.get_or_create_container(key, loro::LoroList::new())?;

        for item in self {
            if let Some(value) = item.to_loro_value() {
                list.push(value)?;
            } else {
                let nested_map = list.push_container(loro::LoroMap::new())?;
                item.reconcile(&nested_map)?;
            }
        }

        Ok(())
    }
}

impl<V: Reconcile> Reconcile for BTreeMap<String, V> {
    fn reconcile(&self, map: &LoroMap) -> Result<(), ReconcileError> {
        for (k, v) in self {
            v.reconcile_field(map, k)?;
        }
        Ok(())
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        let nested_map = map.get_or_create_container(key, loro::LoroMap::new())?;
        self.reconcile(&nested_map)
    }
}

impl<V: Reconcile> Reconcile for BTreeMap<SmolStr, V> {
    fn reconcile(&self, map: &LoroMap) -> Result<(), ReconcileError> {
        for (k, v) in self {
            v.reconcile_field(map, k.as_str())?;
        }
        Ok(())
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        let nested_map = map.get_or_create_container(key, loro::LoroMap::new())?;
        self.reconcile(&nested_map)
    }
}

impl Reconcile for SmolStr {
    fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
        Err(ReconcileError::Custom(
            "SmolStr cannot be reconciled as a root container".into(),
        ))
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        map.insert(key, self.as_str())?;
        Ok(())
    }

    fn to_loro_value(&self) -> Option<LoroValue> {
        Some(LoroValue::String(self.as_str().into()))
    }
}

impl Reconcile for f32 {
    fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
        Err(ReconcileError::Custom(
            "f32 cannot be reconciled as a root container".into(),
        ))
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        map.insert(key, f64::from(*self))?;
        Ok(())
    }

    fn to_loro_value(&self) -> Option<LoroValue> {
        Some(LoroValue::Double(f64::from(*self)))
    }
}

impl<const N: usize> Reconcile for [u8; N] {
    fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
        Err(ReconcileError::Custom(
            "[u8; N] cannot be reconciled as a root container".into(),
        ))
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        map.insert(key, self.as_slice())?;
        Ok(())
    }

    fn to_loro_value(&self) -> Option<LoroValue> {
        Some(LoroValue::Binary(self.to_vec().into()))
    }
}

impl<const N: usize> Reconcile for [f64; N] {
    fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
        Err(ReconcileError::Custom(
            "[f64; N] cannot be reconciled as a root container".into(),
        ))
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        let list = loro::LoroList::new();
        for x in self {
            list.push(*x)?;
        }
        map.insert_container(key, list)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use loro::{LoroDoc, LoroValue};
    use rstest::rstest;

    use super::*;
    use crate::Hydrate;

    fn unwrap_value(v: loro::ValueOrContainer) -> LoroValue {
        match v {
            loro::ValueOrContainer::Value(v) => v,
            loro::ValueOrContainer::Container(_) => panic!("expected value, got container"),
        }
    }

    #[rstest]
    fn reconcile_bool() {
        let doc = LoroDoc::new();
        let map = doc.get_map("test");
        true.reconcile_field(&map, "value")
            .expect("reconcile failed");

        let value = unwrap_value(map.get("value").expect("missing value"));
        let result = bool::hydrate(&value).expect("hydrate failed");
        assert!(result);
    }

    #[rstest]
    fn reconcile_i64() {
        let doc = LoroDoc::new();
        let map = doc.get_map("test");
        42i64
            .reconcile_field(&map, "value")
            .expect("reconcile failed");

        let value = unwrap_value(map.get("value").expect("missing value"));
        let result = i64::hydrate(&value).expect("hydrate failed");
        assert_eq!(result, 42);
    }

    #[rstest]
    fn reconcile_string() {
        let doc = LoroDoc::new();
        let map = doc.get_map("test");
        "hello"
            .to_string()
            .reconcile_field(&map, "value")
            .expect("reconcile failed");

        let value = unwrap_value(map.get("value").expect("missing value"));
        let result = String::hydrate(&value).expect("hydrate failed");
        assert_eq!(result, "hello");
    }

    #[rstest]
    fn reconcile_option_some() {
        let doc = LoroDoc::new();
        let map = doc.get_map("test");
        Some("hello".to_string())
            .reconcile_field(&map, "value")
            .expect("reconcile failed");

        let value = unwrap_value(map.get("value").expect("missing value"));
        let result = Option::<String>::hydrate(&value).expect("hydrate failed");
        assert_eq!(result, Some("hello".to_string()));
    }

    #[rstest]
    fn reconcile_option_none() {
        let doc = LoroDoc::new();
        let map = doc.get_map("test");
        Option::<String>::None
            .reconcile_field(&map, "value")
            .expect("reconcile failed");

        let value = unwrap_value(map.get("value").expect("missing value"));
        let result = Option::<String>::hydrate(&value).expect("hydrate failed");
        assert_eq!(result, None);
    }
}
