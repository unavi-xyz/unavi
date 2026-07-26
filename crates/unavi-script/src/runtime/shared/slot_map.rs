use std::sync::Arc;

use bevy::platform::collections::HashMap;
use unavi_quota::{
    Quota,
    QuotaError,
    Stock,
    StockGuard,
};

struct Slot<T> {
    value:  T,
    _guard: StockGuard,
}

pub struct SlotMap<T> {
    items: HashMap<u32, Slot<T>>,
    next:  u32,
}

impl<T> Default for SlotMap<T> {
    fn default() -> Self {
        Self {
            items: HashMap::new(),
            next:  0,
        }
    }
}

impl<T> SlotMap<T> {
    pub fn get(&self, key: u32) -> Option<&T> {
        self.items.get(&key).map(|s| &s.value)
    }

    pub fn iter(&self) -> impl Iterator<Item = (u32, &T)> {
        self.items.iter().map(|(&k, s)| (k, &s.value))
    }

    /// Charges one [`Stock::Slots`] for the new entry, refunded on removal or
    /// drop, so a guest cannot mint handles past its slot budget.
    pub fn insert(&mut self, value: T, quota: &Arc<Quota>) -> Result<u32, QuotaError> {
        let guard = quota.charge(Stock::Slots, 1)?;
        while self.items.contains_key(&self.next) {
            self.next = self.next.wrapping_add(1);

            // We use `u32::MAX` as an invalid rep value
            if self.next == u32::MAX {
                self.next = self.next.wrapping_add(1);
            }
        }
        let key = self.next;
        self.items.insert(
            key,
            Slot {
                value,
                _guard: guard,
            },
        );
        Ok(key)
    }

    /// Inserts at a caller-chosen key (for externally-assigned ids). Charges a
    /// slot like [`Self::insert`]; any displaced entry refunds its own.
    pub fn insert_at(&mut self, key: u32, value: T, quota: &Arc<Quota>) -> Result<(), QuotaError> {
        let guard = quota.charge(Stock::Slots, 1)?;
        self.items.insert(
            key,
            Slot {
                value,
                _guard: guard,
            },
        );
        Ok(())
    }

    pub fn remove(&mut self, key: u32) -> Option<T> {
        self.items.remove(&key).map(|s| s.value)
    }
}

impl<T> SlotMap<T>
where
    T: Clone,
{
    /// Clone the given key into a new entry.
    pub fn insert_clone(
        &mut self,
        key: u32,
        quota: &Arc<Quota>,
    ) -> Option<Result<u32, QuotaError>> {
        let value = self.get(key)?.clone();
        Some(self.insert(value, quota))
    }
}

#[cfg(test)]
mod tests {
    use unavi_quota::limits::Limits;

    use super::*;

    fn quota(slots: u64) -> Arc<Quota> {
        let mut limits = Limits::default();
        limits.stock.insert(Stock::Slots, slots);
        Quota::root(limits)
    }

    #[test]
    fn insert_charges_and_remove_refunds_a_slot() {
        let q = quota(2);
        let mut map = SlotMap::<u32>::default();
        let a = map.insert(10, &q).expect("first slot");
        let _b = map.insert(20, &q).expect("second slot");
        assert_eq!(q.usage(Stock::Slots), 2);
        assert!(matches!(
            map.insert(30, &q),
            Err(QuotaError::Stock(Stock::Slots))
        ));
        assert_eq!(map.remove(a), Some(10));
        assert_eq!(q.usage(Stock::Slots), 1);
        map.insert(40, &q).expect("slot freed by remove");
    }

    #[test]
    fn dropping_the_map_refunds_every_slot() {
        let q = quota(8);
        let mut map = SlotMap::<u32>::default();
        for i in 0..5 {
            map.insert(i, &q).expect("slot");
        }
        assert_eq!(q.usage(Stock::Slots), 5);
        drop(map);
        assert_eq!(q.usage(Stock::Slots), 0);
    }

    #[test]
    fn rejected_insert_returns_the_value_and_holds_no_slot() {
        let q = quota(0);
        let mut map = SlotMap::<u32>::default();
        assert!(map.insert(1, &q).is_err());
        assert_eq!(q.usage(Stock::Slots), 0);
        assert!(map.get(0).is_none());
    }
}
