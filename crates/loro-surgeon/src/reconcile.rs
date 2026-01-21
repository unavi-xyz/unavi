use std::collections::BTreeMap;

use loro::LoroMap;

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
            // For primitives, we need to push directly. For structs, create nested map.
            // This is a simplification - in practice we'd need type-specific handling.
            let nested_map = list.push_container(loro::LoroMap::new())?;
            item.reconcile(&nested_map)?;
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
