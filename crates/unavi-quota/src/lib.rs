use std::{
    collections::HashMap,
    sync::Arc,
    time::Instant,
};

use parking_lot::Mutex;

pub mod limits;
pub mod registry;

use crate::limits::Limits;

/// A countable, releasable resource: held while live, refunded when freed.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Stock {
    Documents,
    KvMemory,
    PortalWatches,
    Prims,
    Receptors,
    Slots,
    WasmMemory,
}

/// A rate-limited action spent from a token bucket that refills over time.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Flow {
    BlobUpload,
    CreateDocument,
    CreatePrim,
    Emit,
    PortalOpen,
    Publish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotaError {
    Flow(Flow),
    Stock(Stock),
}

impl std::fmt::Display for QuotaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Flow(flow) => write!(f, "flow quota exceeded: {flow:?}"),
            Self::Stock(s) => write!(f, "stock quota exceeded: {s:?}"),
        }
    }
}

impl std::error::Error for QuotaError {}

struct Bucket {
    tokens: f64,
    last:   Instant,
}

/// Resource limits plus an optional owner to rolls up into.
pub struct Quota {
    limits:  Limits,
    stock:   Mutex<HashMap<Stock, u64>>,
    buckets: Mutex<HashMap<Flow, Bucket>>,
    owner:   Mutex<Option<Arc<Self>>>,
}

impl Quota {
    #[must_use]
    pub fn new(limits: Limits, owner: Option<Arc<Self>>) -> Arc<Self> {
        Arc::new(Self {
            limits,
            stock: Mutex::default(),
            buckets: Mutex::default(),
            owner: Mutex::new(owner),
        })
    }

    #[must_use]
    pub fn root(limits: Limits) -> Arc<Self> {
        Self::new(limits, None)
    }

    /// An uncapped, owner-less quota for trusted scripts that must keep running
    /// even when shared budgets are exhausted.
    #[must_use]
    pub fn unlimited() -> Arc<Self> {
        Self::new(Limits::default(), None)
    }

    #[must_use]
    pub fn owner(&self) -> Option<Arc<Self>> {
        self.owner.lock().clone()
    }

    /// Charges `n` units of `stock`, returning a guard that refunds on drop.
    pub fn charge(self: &Arc<Self>, stock: Stock, n: u64) -> Result<StockGuard, QuotaError> {
        self.charge_inner(stock, n)?;
        Ok(StockGuard {
            quota: Arc::clone(self),
            stock,
            n,
        })
    }

    fn charge_inner(&self, stock: Stock, n: u64) -> Result<(), QuotaError> {
        let mut map = self.stock.lock();
        let cur = map.entry(stock).or_insert(0);
        let next = cur.saturating_add(n);
        if self.limits.stock.get(&stock).is_some_and(|&max| next > max) {
            return Err(QuotaError::Stock(stock));
        }
        *cur = next;
        drop(map);

        let Some(owner) = self.owner() else {
            return Ok(());
        };
        if let Err(err) = owner.charge_inner(stock, n) {
            self.refund_local(stock, n);
            return Err(err);
        }
        Ok(())
    }

    fn refund_local(&self, stock: Stock, n: u64) {
        let mut map = self.stock.lock();
        if let Some(cur) = map.get_mut(&stock) {
            *cur = cur.saturating_sub(n);
        }
    }

    fn refund(&self, stock: Stock, n: u64) {
        self.refund_local(stock, n);
        if let Some(owner) = self.owner() {
            owner.refund(stock, n);
        }
    }

    /// Adds standing stock without enforcing caps, for moving already-held
    /// resources to a new owner during [`Self::set_owner`].
    fn adopt(&self, stock: Stock, n: u64) {
        let mut map = self.stock.lock();
        let cur = map.entry(stock).or_insert(0);
        *cur = cur.saturating_add(n);
        drop(map);

        if let Some(owner) = self.owner() {
            owner.adopt(stock, n);
        }
    }

    /// Repoints this quota at a new owner, moving its standing stock from the
    /// old owner to the new so an ownership change leaks neither cap.
    pub fn set_owner(&self, new_owner: Option<Arc<Self>>) {
        let mut slot = self.owner.lock();
        let same = match (slot.as_ref(), new_owner.as_ref()) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        if same {
            return;
        }
        let held = self
            .stock
            .lock()
            .iter()
            .filter(|&(_, &n)| n > 0)
            .map(|(&s, &n)| (s, n))
            .collect::<Vec<_>>();
        if let Some(old) = slot.as_ref() {
            for &(stock, n) in &held {
                old.refund(stock, n);
            }
        }
        if let Some(new) = new_owner.as_ref() {
            for &(stock, n) in &held {
                new.adopt(stock, n);
            }
        }
        *slot = new_owner;
    }

    /// Spends `n` tokens of `flow`, refilling each bucket by elapsed time
    /// first.
    pub fn spend(&self, flow: Flow, n: f64) -> Result<(), QuotaError> {
        self.spend_inner(flow, n, Instant::now())
    }

    fn spend_inner(&self, flow: Flow, n: f64, now: Instant) -> Result<(), QuotaError> {
        if let Some(limit) = self.limits.flow.get(&flow).copied() {
            let mut buckets = self.buckets.lock();
            let bucket = buckets.entry(flow).or_insert(Bucket {
                tokens: limit.capacity,
                last:   now,
            });
            let elapsed = now.saturating_duration_since(bucket.last).as_secs_f64();
            bucket.tokens = elapsed
                .mul_add(limit.refill_per_sec, bucket.tokens)
                .min(limit.capacity);
            bucket.last = now;
            if bucket.tokens < n {
                return Err(QuotaError::Flow(flow));
            }
            bucket.tokens -= n;
            drop(buckets);
        }
        let Some(owner) = self.owner() else {
            return Ok(());
        };
        if let Err(err) = owner.spend_inner(flow, n, now) {
            self.refill_local(flow, n);
            return Err(err);
        }
        Ok(())
    }

    fn refill_local(&self, flow: Flow, n: f64) {
        if self.limits.flow.contains_key(&flow) {
            let mut buckets = self.buckets.lock();
            if let Some(bucket) = buckets.get_mut(&flow) {
                bucket.tokens += n;
            }
        }
    }

    /// Charges without a guard; caller must pair it with a [`Self::release`].
    /// For resources tracked elsewhere, like wasm memory growth or KV bytes.
    pub fn try_charge(&self, stock: Stock, n: u64) -> Result<(), QuotaError> {
        self.charge_inner(stock, n)
    }

    /// Charges `n` units of `stock`, returning a resizable [`StockHold`] that
    /// refunds whatever it holds on drop. Use for data whose footprint changes
    /// over its lifetime, like a KV cell.
    pub fn hold(self: &Arc<Self>, stock: Stock, n: u64) -> Result<StockHold, QuotaError> {
        self.charge_inner(stock, n)?;
        Ok(StockHold {
            quota: Arc::clone(self),
            stock,
            n,
        })
    }

    pub fn release(&self, stock: Stock, n: u64) {
        self.refund(stock, n);
    }

    #[must_use]
    pub fn usage(&self, stock: Stock) -> u64 {
        self.stock.lock().get(&stock).copied().unwrap_or(0)
    }
}

/// Holds a stock charge for as long as the guarded resource lives.
#[must_use = "dropping the guard immediately refunds the charge"]
pub struct StockGuard {
    quota: Arc<Quota>,
    stock: Stock,
    n:     u64,
}

impl Drop for StockGuard {
    fn drop(&mut self) {
        self.quota.refund(self.stock, self.n);
    }
}

/// A resizable stock hold that refunds whatever it currently holds on drop.
///
/// [`Self::resize`] charges only the positive delta when growing and refunds
/// when shrinking, so an overwrite that does not grow always succeeds even at a
/// full cap.
#[must_use = "dropping the hold immediately refunds the held stock"]
pub struct StockHold {
    quota: Arc<Quota>,
    stock: Stock,
    n:     u64,
}

impl StockHold {
    /// Adjusts the held amount to `new_n`. On growth the delta is charged and
    /// may fail (leaving the hold unchanged); on shrink it always succeeds.
    pub fn resize(&mut self, new_n: u64) -> Result<(), QuotaError> {
        if new_n > self.n {
            self.quota.charge_inner(self.stock, new_n - self.n)?;
        } else {
            self.quota.refund(self.stock, self.n - new_n);
        }
        self.n = new_n;
        Ok(())
    }

    #[must_use]
    pub const fn held(&self) -> u64 {
        self.n
    }
}

impl Drop for StockHold {
    fn drop(&mut self) {
        self.quota.refund(self.stock, self.n);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        limits::{
            FlowLimit,
            Limits,
        },
        *,
    };

    fn limits_stock(stock: Stock, max: u64) -> Limits {
        let mut l = Limits::default();
        l.stock.insert(stock, max);
        l
    }

    #[test]
    fn stock_charges_and_refunds() {
        let q = Quota::root(limits_stock(Stock::Prims, 2));
        let a = q.charge(Stock::Prims, 1).expect("first");
        let _b = q.charge(Stock::Prims, 1).expect("second");
        assert_eq!(q.usage(Stock::Prims), 2);
        assert!(matches!(
            q.charge(Stock::Prims, 1),
            Err(QuotaError::Stock(Stock::Prims))
        ));
        drop(a);
        assert_eq!(q.usage(Stock::Prims), 1);
        let _g = q.charge(Stock::Prims, 1).expect("after refund");
    }

    #[test]
    fn unset_stock_is_unbounded() {
        let q = Quota::root(Limits::default());
        let _g = q.charge(Stock::Prims, u64::MAX).expect("unbounded");
    }

    #[test]
    fn charge_rolls_up_to_owner() {
        let owner = Quota::root(limits_stock(Stock::Documents, 1));
        let doc = Quota::new(Limits::default(), Some(Arc::clone(&owner)));
        let _g = doc.charge(Stock::Documents, 1).expect("within owner cap");
        assert_eq!(owner.usage(Stock::Documents), 1);
        assert!(matches!(
            doc.charge(Stock::Documents, 1),
            Err(QuotaError::Stock(Stock::Documents))
        ));
    }

    #[test]
    fn failed_owner_charge_does_not_leak_into_document() {
        let owner = Quota::root(limits_stock(Stock::Documents, 1));
        let doc = Quota::new(limits_stock(Stock::Documents, 5), Some(Arc::clone(&owner)));
        let _g = doc.charge(Stock::Documents, 1).expect("ok");
        assert!(doc.charge(Stock::Documents, 1).is_err());
        assert_eq!(doc.usage(Stock::Documents), 1, "no phantom doc charge");
        assert_eq!(owner.usage(Stock::Documents), 1);
    }

    #[test]
    fn owner_refund_rolls_up() {
        let owner = Quota::root(limits_stock(Stock::Prims, 4));
        let doc = Quota::new(Limits::default(), Some(Arc::clone(&owner)));
        let g = doc.charge(Stock::Prims, 2).expect("ok");
        assert_eq!(owner.usage(Stock::Prims), 2);
        drop(g);
        assert_eq!(owner.usage(Stock::Prims), 0);
    }

    #[test]
    fn set_owner_migrates_standing_stock() {
        let old = Quota::root(limits_stock(Stock::Prims, 10));
        let new = Quota::root(limits_stock(Stock::Prims, 10));
        let doc = Quota::new(Limits::default(), Some(Arc::clone(&old)));
        let g = doc.charge(Stock::Prims, 3).expect("ok");
        assert_eq!(old.usage(Stock::Prims), 3);

        doc.set_owner(Some(Arc::clone(&new)));
        assert_eq!(old.usage(Stock::Prims), 0, "old owner released");
        assert_eq!(new.usage(Stock::Prims), 3, "new owner adopted");

        drop(g);
        assert_eq!(new.usage(Stock::Prims), 0, "refund follows the new owner");
    }

    #[test]
    fn hold_grows_shrinks_and_refunds_on_drop() {
        let q = Quota::root(limits_stock(Stock::KvMemory, 100));
        let mut hold = q.hold(Stock::KvMemory, 40).expect("initial");
        assert_eq!(q.usage(Stock::KvMemory), 40);

        hold.resize(90).expect("grow within cap");
        assert_eq!(q.usage(Stock::KvMemory), 90);

        assert!(
            hold.resize(120).is_err(),
            "growth past the cap fails and leaves the hold unchanged"
        );
        assert_eq!(hold.held(), 90);
        assert_eq!(q.usage(Stock::KvMemory), 90);

        drop(hold);
        assert_eq!(q.usage(Stock::KvMemory), 0, "drop refunds the full hold");
    }

    /// A shrink must always succeed, even when the cap is otherwise exhausted —
    /// the basis for overwriting a large KV value with a small one when full.
    #[test]
    fn hold_shrink_succeeds_at_full_cap() {
        let q = Quota::root(limits_stock(Stock::KvMemory, 50));
        let mut hold = q.hold(Stock::KvMemory, 50).expect("fill the cap");
        assert!(q.hold(Stock::KvMemory, 1).is_err(), "cap is full");

        hold.resize(10).expect("shrink frees stock");
        assert_eq!(q.usage(Stock::KvMemory), 10);
        let _room = q.hold(Stock::KvMemory, 40).expect("freed room is reusable");
    }

    #[test]
    fn hold_rolls_up_and_refunds_to_owner() {
        let owner = Quota::root(limits_stock(Stock::KvMemory, 100));
        let doc = Quota::new(Limits::default(), Some(Arc::clone(&owner)));
        let mut hold = doc.hold(Stock::KvMemory, 30).expect("ok");
        assert_eq!(owner.usage(Stock::KvMemory), 30);
        hold.resize(10).expect("shrink");
        assert_eq!(owner.usage(Stock::KvMemory), 10);
        drop(hold);
        assert_eq!(owner.usage(Stock::KvMemory), 0);
    }

    fn limits_flow(flow: Flow, capacity: f64, refill_per_sec: f64) -> Limits {
        let mut l = Limits::default();
        l.flow.insert(
            flow,
            FlowLimit {
                capacity,
                refill_per_sec,
            },
        );
        l
    }

    #[test]
    fn flow_drains_then_refills() {
        let q = Quota::root(limits_flow(Flow::PortalOpen, 2.0, 1.0));
        let t0 = Instant::now();
        q.spend_inner(Flow::PortalOpen, 1.0, t0).expect("1");
        q.spend_inner(Flow::PortalOpen, 1.0, t0).expect("2");
        assert!(q.spend_inner(Flow::PortalOpen, 1.0, t0).is_err());
        let t1 = t0 + Duration::from_secs(1);
        q.spend_inner(Flow::PortalOpen, 1.0, t1).expect("refilled");
    }

    #[test]
    fn flow_refill_caps_at_capacity() {
        let q = Quota::root(limits_flow(Flow::Emit, 4.0, 1000.0));
        let t0 = Instant::now();
        q.spend_inner(Flow::Emit, 4.0, t0).expect("drain");
        let t1 = t0 + Duration::from_secs(10);
        q.spend_inner(Flow::Emit, 4.0, t1).expect("full again");
        assert!(q.spend_inner(Flow::Emit, 1.0, t1).is_err());
    }

    #[test]
    fn failed_owner_spend_refunds_self() {
        let owner = Quota::root(limits_flow(Flow::Emit, 1.0, 0.0));
        let doc = Quota::new(limits_flow(Flow::Emit, 10.0, 0.0), Some(Arc::clone(&owner)));
        let t0 = Instant::now();
        doc.spend_inner(Flow::Emit, 1.0, t0).expect("first ok");
        assert!(
            doc.spend_inner(Flow::Emit, 1.0, t0).is_err(),
            "owner exhausted"
        );
        assert_eq!(
            doc.buckets.lock().get(&Flow::Emit).map(|b| b.tokens),
            Some(9.0),
        );
    }
}
