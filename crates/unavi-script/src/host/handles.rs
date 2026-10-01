//! Guest-visible handles to host values, each charged against the script's
//! handle quota.

use std::sync::Arc;

use bevy::platform::collections::HashMap;
use unavi_policy::quota::{
    Quota,
    QuotaError,
    Stock,
    StockLease,
};

use crate::error::ScriptError;

struct Slot<T> {
    value:  T,
    _lease: StockLease,
}

/// Values a guest refers to by `u32` handle.
///
/// Every entry holds one [`Stock::Slots`] unit until it is removed or the
/// table drops, so a guest cannot mint handles past its budget. Keys advance
/// past every one issued before wrapping, so a freed handle is not reissued
/// until the counter comes round again.
pub struct HandleTable<T> {
    items: HashMap<u32, Slot<T>>,
    next:  u32,
}

impl<T> Default for HandleTable<T> {
    fn default() -> Self {
        Self {
            items: HashMap::new(),
            next:  0,
        }
    }
}

impl<T> HandleTable<T> {
    /// [`ScriptError::InvalidHandle`] for a handle not in the table.
    pub fn get(&self, key: u32) -> Result<&T, ScriptError> {
        self.items
            .get(&key)
            .map(|slot| &slot.value)
            .ok_or(ScriptError::InvalidHandle)
    }

    pub fn get_mut(&mut self, key: u32) -> Result<&mut T, ScriptError> {
        self.items
            .get_mut(&key)
            .map(|slot| &mut slot.value)
            .ok_or(ScriptError::InvalidHandle)
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.items.values().map(|slot| &slot.value)
    }

    pub fn insert(&mut self, value: T, quota: &Arc<Quota>) -> Result<u32, QuotaError> {
        let lease = quota.lease(Stock::Slots, 1)?;
        while self.items.contains_key(&self.next) {
            self.next = self.next.wrapping_add(1);
        }
        let key = self.next;
        self.next = self.next.wrapping_add(1);
        self.items.insert(
            key,
            Slot {
                value,
                _lease: lease,
            },
        );
        Ok(key)
    }

    pub fn remove(&mut self, key: u32) -> Result<T, ScriptError> {
        self.items
            .remove(&key)
            .map(|slot| slot.value)
            .ok_or(ScriptError::InvalidHandle)
    }
}

#[cfg(test)]
mod tests {
    use unavi_policy::quota::limits::Limits;

    use super::*;

    fn quota(slots: u64) -> Arc<Quota> {
        let mut limits = Limits::default();
        limits.stock.insert(Stock::Slots, slots);
        Quota::new(limits, None)
    }

    #[test]
    fn insert_charges_and_remove_refunds_a_handle() {
        let q = quota(2);
        let mut table = HandleTable::<u32>::default();
        let a = table.insert(10, &q).expect("first handle");
        let _b = table.insert(20, &q).expect("second handle");
        assert_eq!(q.usage(Stock::Slots), 2);
        assert!(matches!(
            table.insert(30, &q),
            Err(QuotaError::Stock(Stock::Slots))
        ));
        assert_eq!(table.remove(a), Ok(10));
        assert_eq!(q.usage(Stock::Slots), 1);
        table.insert(40, &q).expect("handle freed by remove");
    }

    #[test]
    fn a_freed_handle_is_not_reissued_at_once() {
        let q = quota(8);
        let mut table = HandleTable::<u32>::default();
        let a = table.insert(1, &q).expect("handle");
        table.remove(a).expect("remove");
        let b = table.insert(2, &q).expect("handle");
        assert_ne!(a, b, "a cached stale handle must not name the new value");
    }

    #[test]
    fn dropping_the_table_refunds_every_handle() {
        let q = quota(8);
        let mut table = HandleTable::<u32>::default();
        for i in 0..5 {
            table.insert(i, &q).expect("handle");
        }
        drop(table);
        assert_eq!(q.usage(Stock::Slots), 0);
    }

    #[test]
    fn an_unknown_handle_is_invalid() {
        let table = HandleTable::<u32>::default();
        assert_eq!(table.get(7), Err(ScriptError::InvalidHandle));
    }
}
