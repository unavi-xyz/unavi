use std::{
    collections::HashMap,
    sync::Arc,
    time::Instant,
};

use parking_lot::Mutex;

#[cfg(not(target_family = "wasm"))] pub mod limiter;
pub mod limits;
pub mod registry;

use crate::quota::limits::Limits;

/// A countable, releasable resource: held while live, refunded when freed.
/// Charges roll up the scope chain, so a per-document cap and the enclosing
/// space and owner caps are honored at once.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Stock {
    WasmBytes,
    Documents,
    Prims,
    Slots,
    PortalWatches,
    Receptors,
}

/// A rate-limited action: spent from a token bucket that refills over time.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Flow {
    CreateDocument,
    CreatePrim,
    PortalOpen,
    Emit,
    Publish,
    BlobUpload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotaError {
    Stock(Stock),
    Flow(Flow),
}

struct Bucket {
    tokens: f64,
    last:   Instant,
}

/// One scope in the document / space / user lattice.
///
/// A document charges into both its space and its owning user, so a node may
/// have several parents. Charges and spends recurse into every parent and roll
/// back on the way out if any refuses, so partial charges never linger.
pub struct Quota {
    limits:  Limits,
    stock:   Mutex<HashMap<Stock, u64>>,
    buckets: Mutex<HashMap<Flow, Bucket>>,
    parents: Vec<Arc<Self>>,
}

impl Quota {
    #[must_use]
    pub fn new(limits: Limits, parents: Vec<Arc<Self>>) -> Arc<Self> {
        Arc::new(Self {
            limits,
            stock: Mutex::new(HashMap::new()),
            buckets: Mutex::new(HashMap::new()),
            parents,
        })
    }

    #[must_use]
    pub fn root(limits: Limits) -> Arc<Self> {
        Self::new(limits, Vec::new())
    }

    #[must_use]
    pub fn child(self: &Arc<Self>, limits: Limits) -> Arc<Self> {
        Self::new(limits, vec![Arc::clone(self)])
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

        for (i, parent) in self.parents.iter().enumerate() {
            if let Err(err) = parent.charge_inner(stock, n) {
                for done in &self.parents[..i] {
                    done.refund(stock, n);
                }
                self.refund_local(stock, n);
                return Err(err);
            }
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
        for parent in &self.parents {
            parent.refund(stock, n);
        }
    }

    /// Spends `n` tokens of `flow`, refilling each bucket by elapsed time
    /// first.
    pub fn spend(&self, flow: Flow, n: f64) -> Result<(), QuotaError> {
        self.spend_inner(flow, n, Instant::now())
    }

    fn spend_inner(&self, flow: Flow, n: f64, now: Instant) -> Result<(), QuotaError> {
        let limit = self.limits.flow.get(&flow).copied();
        if let Some(limit) = limit {
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
        for (i, parent) in self.parents.iter().enumerate() {
            if let Err(err) = parent.spend_inner(flow, n, now) {
                self.refill_local(flow, n);
                for done in &self.parents[..i] {
                    done.refill(flow, n);
                }
                return Err(err);
            }
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

    fn refill(&self, flow: Flow, n: f64) {
        self.refill_local(flow, n);
        for parent in &self.parents {
            parent.refill(flow, n);
        }
    }

    /// Charges without a guard. The caller is responsible for a matching
    /// [`Self::release`]; used where the lifetime is tracked elsewhere, such as
    /// linear-memory growth inside the wasm [`limiter`].
    pub fn try_charge(&self, stock: Stock, n: u64) -> Result<(), QuotaError> {
        self.charge_inner(stock, n)
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

/// Carries stock guards on a spawned entity, so despawning the entity refunds
/// the charges it held (e.g. a child document or a portal watch).
#[derive(bevy::prelude::Component, Default)]
pub struct QuotaGuards(pub Vec<StockGuard>);

impl Drop for StockGuard {
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
    fn child_charge_rolls_up_to_parent() {
        let parent = Quota::root(limits_stock(Stock::Documents, 1));
        let child = parent.child(Limits::default());
        let _g = child
            .charge(Stock::Documents, 1)
            .expect("within parent cap");
        assert_eq!(parent.usage(Stock::Documents), 1);
        assert!(matches!(
            child.charge(Stock::Documents, 1),
            Err(QuotaError::Stock(Stock::Documents))
        ));
    }

    #[test]
    fn failed_child_charge_does_not_leak_into_parent() {
        let parent = Quota::root(limits_stock(Stock::Documents, 5));
        let child = parent.child(limits_stock(Stock::Documents, 1));
        let _g = child.charge(Stock::Documents, 1).expect("ok");
        assert!(child.charge(Stock::Documents, 1).is_err());
        assert_eq!(
            parent.usage(Stock::Documents),
            1,
            "no phantom parent charge"
        );
    }

    #[test]
    fn charge_rolls_into_every_parent_and_rolls_back_cleanly() {
        let space = Quota::root(limits_stock(Stock::Prims, 10));
        let user = Quota::root(limits_stock(Stock::Prims, 1));
        let doc = Quota::new(
            Limits::default(),
            vec![Arc::clone(&space), Arc::clone(&user)],
        );

        let _g = doc.charge(Stock::Prims, 1).expect("within both parents");
        assert_eq!(space.usage(Stock::Prims), 1);
        assert_eq!(user.usage(Stock::Prims), 1);

        // The user cap (1) is the binding constraint; the failed charge must
        // leave no residue in the space it already touched.
        assert!(doc.charge(Stock::Prims, 1).is_err());
        assert_eq!(space.usage(Stock::Prims), 1, "space rolled back");
        assert_eq!(user.usage(Stock::Prims), 1);
    }

    #[test]
    fn child_refund_rolls_up() {
        let parent = Quota::root(limits_stock(Stock::Prims, 4));
        let child = parent.child(Limits::default());
        let g = child.charge(Stock::Prims, 2).expect("ok");
        assert_eq!(parent.usage(Stock::Prims), 2);
        drop(g);
        assert_eq!(parent.usage(Stock::Prims), 0);
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
    fn failed_child_spend_refunds_self() {
        let parent = Quota::root(limits_flow(Flow::Emit, 1.0, 0.0));
        let child = parent.child(limits_flow(Flow::Emit, 10.0, 0.0));
        let t0 = Instant::now();
        child.spend_inner(Flow::Emit, 1.0, t0).expect("first ok");
        assert!(
            child.spend_inner(Flow::Emit, 1.0, t0).is_err(),
            "parent exhausted"
        );
        // The failed attempt must not have spent the child's own token.
        assert_eq!(
            child.buckets.lock().get(&Flow::Emit).map(|b| b.tokens),
            Some(9.0),
        );
    }
}
