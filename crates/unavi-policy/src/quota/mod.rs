//! Resource accounting.
//!
//! A [`Stock`] is charged and refunded. A [`Flow`] is spent from a token bucket
//! that refills over time. A [`Quota`] may roll up into an owner, charging
//! both.

use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};

use parking_lot::Mutex;
use web_time::Instant;

use crate::quota::limits::{
    FlowLimit,
    Limits,
};

pub mod limits;

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

impl Stock {
    const ALL: [Self; 7] = [
        Self::Documents,
        Self::KvMemory,
        Self::PortalWatches,
        Self::Prims,
        Self::Receptors,
        Self::Slots,
        Self::WasmMemory,
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Flow {
    BlobUpload,
    CreateDocument,
    CreatePrim,
    Emit,
    PortalOpen,
    SyncDoc,
}

impl Flow {
    const ALL: [Self; 6] = [
        Self::BlobUpload,
        Self::CreateDocument,
        Self::CreatePrim,
        Self::Emit,
        Self::PortalOpen,
        Self::SyncDoc,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum QuotaError {
    #[error("flow quota exceeded: {0:?}")]
    Flow(Flow),
    #[error("stock quota exceeded: {0:?}")]
    Stock(Stock),
}

/// What the chain can do about a flow request now. Ordered worst-last, so
/// combining levels is a `max`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Reservation {
    Ready,
    After(Duration),
    /// The ask exceeds some level's whole capacity, so no wait satisfies it.
    Never,
}

struct Bucket {
    tokens: f64,
    last:   Instant,
}

impl Bucket {
    const fn full(limit: FlowLimit, now: Instant) -> Self {
        Self {
            tokens: limit.capacity,
            last:   now,
        }
    }

    /// Adds the tokens elapsed time has earned, stopping at capacity.
    fn refill(&mut self, limit: FlowLimit, now: Instant) {
        let elapsed = now.saturating_duration_since(self.last).as_secs_f64();
        self.tokens = elapsed
            .mul_add(limit.refill_per_sec, self.tokens)
            .min(limit.capacity);
        self.last = now;
    }
}

/// One scope's caps, and the quota its charges roll up into.
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

    /// An uncapped, owner-less quota, for trusted scripts.
    #[must_use]
    pub fn unlimited() -> Arc<Self> {
        Self::new(Limits::default(), None)
    }

    pub(crate) fn owner(&self) -> Option<Arc<Self>> {
        self.owner.lock().clone()
    }

    #[must_use]
    pub fn usage(&self, stock: Stock) -> u64 {
        self.stock.lock().get(&stock).copied().unwrap_or(0)
    }

    /// Charges `n` units of `stock` at every level. Pair with
    /// [`Self::release`].
    pub fn charge(&self, stock: Stock, n: u64) -> Result<(), QuotaError> {
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
        if let Err(err) = owner.charge(stock, n) {
            self.release_local(stock, n);
            return Err(err);
        }
        Ok(())
    }

    /// Charges `n` units of `stock`, returning a hold that refunds on drop.
    pub fn hold(self: &Arc<Self>, stock: Stock, n: u64) -> Result<StockHold, QuotaError> {
        self.charge(stock, n)?;
        Ok(StockHold {
            quota: Arc::clone(self),
            stock,
            n,
        })
    }

    pub fn release(&self, stock: Stock, n: u64) {
        self.release_local(stock, n);
        if let Some(owner) = self.owner() {
            owner.release(stock, n);
        }
    }

    fn release_local(&self, stock: Stock, n: u64) {
        let mut map = self.stock.lock();
        if let Some(cur) = map.get_mut(&stock) {
            *cur = cur.saturating_sub(n);
        }
    }

    /// Adds standing stock without enforcing caps, for [`Self::set_owner`].
    fn adopt(&self, stock: Stock, n: u64) {
        let mut map = self.stock.lock();
        let cur = map.entry(stock).or_insert(0);
        *cur = cur.saturating_add(n);
        drop(map);

        if let Some(owner) = self.owner() {
            owner.adopt(stock, n);
        }
    }

    /// Reads the whole owner chain. Nothing is taken until [`Self::commit`].
    #[must_use]
    pub fn reserve(&self, flow: Flow, n: f64) -> Reservation {
        self.reserve_inner(flow, n, Instant::now())
    }

    fn reserve_inner(&self, flow: Flow, n: f64, now: Instant) -> Reservation {
        let here = self.reserve_local(flow, n, now);
        let Some(owner) = self.owner() else {
            return here;
        };
        here.max(owner.reserve_inner(flow, n, now))
    }

    fn reserve_local(&self, flow: Flow, n: f64, now: Instant) -> Reservation {
        let Some(limit) = self.limits.flow.get(&flow).copied() else {
            return Reservation::Ready;
        };
        if n > limit.capacity {
            return Reservation::Never;
        }

        let mut buckets = self.buckets.lock();
        let bucket = buckets
            .entry(flow)
            .or_insert_with(|| Bucket::full(limit, now));
        // Refilling mutates the bucket but takes nothing.
        bucket.refill(limit, now);
        let tokens = bucket.tokens;
        drop(buckets);

        if tokens >= n {
            Reservation::Ready
        } else if limit.refill_per_sec <= 0.0 {
            Reservation::Never
        } else {
            Reservation::After(Duration::from_secs_f64((n - tokens) / limit.refill_per_sec))
        }
    }

    /// Takes `n` at every level that caps `flow`. Only sound immediately after
    /// a [`Reservation::Ready`], which anything else may drive negative.
    pub fn commit(&self, flow: Flow, n: f64) {
        self.commit_inner(flow, n, Instant::now());
    }

    fn commit_inner(&self, flow: Flow, n: f64, now: Instant) {
        if let Some(limit) = self.limits.flow.get(&flow).copied() {
            // A bucket that does not exist yet is a full one, not a free one.
            let mut buckets = self.buckets.lock();
            buckets
                .entry(flow)
                .or_insert_with(|| Bucket::full(limit, now))
                .tokens -= n;
        }
        if let Some(owner) = self.owner() {
            owner.commit_inner(flow, n, now);
        }
    }

    /// Repoints this quota at a new owner, moving its standing stock across.
    /// Refuses an owner that already rolls up into this quota.
    pub(crate) fn set_owner(self: &Arc<Self>, new_owner: Option<Arc<Self>>) {
        if let Some(new) = new_owner.as_ref()
            && (Arc::ptr_eq(new, self) || new.rolls_up_into(self))
        {
            return;
        }

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
                old.release(stock, n);
            }
        }
        if let Some(new) = new_owner.as_ref() {
            for &(stock, n) in &held {
                new.adopt(stock, n);
            }
        }
        *slot = new_owner;
    }

    /// Whether `other` is anything this quota rolls up into.
    fn rolls_up_into(&self, other: &Arc<Self>) -> bool {
        let mut current = self.owner();
        while let Some(quota) = current {
            if Arc::ptr_eq(&quota, other) {
                return true;
            }
            current = quota.owner();
        }
        false
    }
}

/// A stock charge that refunds what it holds on drop.
#[must_use = "dropping the hold immediately refunds the held stock"]
pub struct StockHold {
    quota: Arc<Quota>,
    stock: Stock,
    n:     u64,
}

impl StockHold {
    /// Adjusts the held amount to `new_n`. Growing charges the delta and may
    /// fail, leaving the hold unchanged. Shrinking always succeeds.
    pub fn resize(&mut self, new_n: u64) -> Result<(), QuotaError> {
        if new_n > self.n {
            self.quota.charge(self.stock, new_n - self.n)?;
        } else {
            self.quota.release(self.stock, self.n - new_n);
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
        self.quota.release(self.stock, self.n);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits_stock(stock: Stock, max: u64) -> Limits {
        let mut limits = Limits::default();
        limits.stock.insert(stock, max);
        limits
    }

    fn limits_flow(flow: Flow, capacity: f64, refill_per_sec: f64) -> Limits {
        let mut limits = Limits::default();
        limits.flow.insert(
            flow,
            FlowLimit {
                capacity,
                refill_per_sec,
            },
        );
        limits
    }

    #[test]
    fn a_cap_is_enforced_and_freed_again() {
        let q = Quota::new(limits_stock(Stock::Prims, 2), None);
        let first = q.hold(Stock::Prims, 1).expect("first");
        let _second = q.hold(Stock::Prims, 1).expect("second");
        assert_eq!(q.usage(Stock::Prims), 2);
        assert!(matches!(
            q.hold(Stock::Prims, 1),
            Err(QuotaError::Stock(Stock::Prims))
        ));

        drop(first);
        assert_eq!(q.usage(Stock::Prims), 1);
        let _third = q.hold(Stock::Prims, 1).expect("after the refund");
    }

    #[test]
    fn unset_stock_is_unbounded() {
        let q = Quota::new(Limits::default(), None);
        let _hold = q.hold(Stock::Prims, u64::MAX).expect("unbounded");
    }

    #[test]
    fn a_failed_owner_charge_leaves_nothing_behind() {
        let owner = Quota::new(limits_stock(Stock::Documents, 1), None);
        let doc = Quota::new(limits_stock(Stock::Documents, 5), Some(Arc::clone(&owner)));

        let _hold = doc.hold(Stock::Documents, 1).expect("within the owner cap");
        assert!(
            doc.hold(Stock::Documents, 1).is_err(),
            "the owner cap governs even where the document has room"
        );
        assert_eq!(doc.usage(Stock::Documents), 1, "no phantom document charge");
        assert_eq!(owner.usage(Stock::Documents), 1);
    }

    #[test]
    fn set_owner_migrates_standing_stock() {
        let old = Quota::new(limits_stock(Stock::Prims, 10), None);
        let new = Quota::new(limits_stock(Stock::Prims, 10), None);
        let doc = Quota::new(Limits::default(), Some(Arc::clone(&old)));
        let hold = doc.hold(Stock::Prims, 3).expect("charge");
        assert_eq!(old.usage(Stock::Prims), 3);

        doc.set_owner(Some(Arc::clone(&new)));
        assert_eq!(old.usage(Stock::Prims), 0, "old owner released");
        assert_eq!(new.usage(Stock::Prims), 3, "new owner adopted");

        drop(hold);
        assert_eq!(new.usage(Stock::Prims), 0, "refund follows the new owner");
    }

    #[test]
    fn a_hold_grows_shrinks_and_refunds_on_drop() {
        let q = Quota::new(limits_stock(Stock::KvMemory, 100), None);
        let mut hold = q.hold(Stock::KvMemory, 40).expect("initial");
        assert_eq!(q.usage(Stock::KvMemory), 40);

        hold.resize(90).expect("grow within the cap");
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

    #[test]
    fn a_hold_shrinks_at_a_full_cap() {
        let q = Quota::new(limits_stock(Stock::KvMemory, 50), None);
        let mut hold = q.hold(Stock::KvMemory, 50).expect("fill the cap");
        assert!(q.hold(Stock::KvMemory, 1).is_err(), "cap is full");

        hold.resize(10).expect("shrink frees stock");
        assert_eq!(q.usage(Stock::KvMemory), 10);
        let _room = q.hold(Stock::KvMemory, 40).expect("freed room is reusable");
    }

    #[test]
    fn a_hold_rolls_up_and_refunds_to_its_owner() {
        let owner = Quota::new(limits_stock(Stock::KvMemory, 100), None);
        let doc = Quota::new(Limits::default(), Some(Arc::clone(&owner)));
        let mut hold = doc.hold(Stock::KvMemory, 30).expect("charge");
        assert_eq!(owner.usage(Stock::KvMemory), 30);

        hold.resize(10).expect("shrink");
        assert_eq!(owner.usage(Stock::KvMemory), 10);

        drop(hold);
        assert_eq!(owner.usage(Stock::KvMemory), 0);
    }

    #[test]
    fn a_bucket_drains_then_refills() {
        let q = Quota::new(limits_flow(Flow::PortalOpen, 2.0, 1.0), None);
        let t0 = Instant::now();

        for _ in 0..2 {
            assert_eq!(
                q.reserve_inner(Flow::PortalOpen, 1.0, t0),
                Reservation::Ready
            );
            q.commit_inner(Flow::PortalOpen, 1.0, t0);
        }
        assert!(matches!(
            q.reserve_inner(Flow::PortalOpen, 1.0, t0),
            Reservation::After(_)
        ));
        assert_eq!(
            q.reserve_inner(Flow::PortalOpen, 1.0, t0 + Duration::from_secs(1)),
            Reservation::Ready
        );
    }

    #[test]
    fn a_refill_stops_at_capacity() {
        let q = Quota::new(limits_flow(Flow::Emit, 4.0, 1_000.0), None);
        let t0 = Instant::now();
        q.commit_inner(Flow::Emit, 4.0, t0);

        let t1 = t0 + Duration::from_secs(10);
        assert_eq!(q.reserve_inner(Flow::Emit, 4.0, t1), Reservation::Ready);
        q.commit_inner(Flow::Emit, 4.0, t1);
        assert!(
            matches!(
                q.reserve_inner(Flow::Emit, 1.0, t1),
                Reservation::After(_) | Reservation::Never
            ),
            "ten seconds of refill must still leave only one bucketful"
        );
    }

    #[test]
    fn a_reservation_takes_nothing_until_it_commits() {
        let q = Quota::new(limits_flow(Flow::Emit, 10.0, 1.0), None);
        let t0 = Instant::now();

        for _ in 0..3 {
            assert_eq!(
                q.reserve_inner(Flow::Emit, 10.0, t0),
                Reservation::Ready,
                "peeking must not drain the bucket"
            );
        }

        q.commit_inner(Flow::Emit, 10.0, t0);
        assert!(matches!(
            q.reserve_inner(Flow::Emit, 10.0, t0),
            Reservation::After(_)
        ));
    }

    #[test]
    fn an_ask_past_capacity_is_refused_rather_than_waited_on() {
        let q = Quota::new(limits_flow(Flow::Emit, 10.0, 1.0), None);
        assert_eq!(
            q.reserve(Flow::Emit, 11.0),
            Reservation::Never,
            "no wait fills a bucket past its own capacity"
        );
    }

    #[test]
    fn the_wait_is_what_the_bucket_needs_to_refill() {
        let q = Quota::new(limits_flow(Flow::Emit, 10.0, 2.0), None);
        let t0 = Instant::now();
        q.commit_inner(Flow::Emit, 10.0, t0);

        assert_eq!(
            q.reserve_inner(Flow::Emit, 4.0, t0),
            Reservation::After(Duration::from_secs(2)),
            "four tokens at two per second is two seconds"
        );
    }

    #[test]
    fn the_slowest_level_of_the_chain_governs() {
        let peer = Quota::new(limits_flow(Flow::Emit, 10.0, 1.0), None);
        let doc = Quota::new(limits_flow(Flow::Emit, 10.0, 10.0), Some(Arc::clone(&peer)));
        let t0 = Instant::now();
        doc.commit_inner(Flow::Emit, 10.0, t0);

        assert_eq!(
            doc.reserve_inner(Flow::Emit, 5.0, t0),
            Reservation::After(Duration::from_secs(5)),
            "the slow peer bucket governs, not the fast document one"
        );
    }

    #[test]
    fn committing_charges_every_level() {
        let peer = Quota::new(limits_flow(Flow::Emit, 10.0, 1.0), None);
        let doc = Quota::new(limits_flow(Flow::Emit, 10.0, 1.0), Some(Arc::clone(&peer)));
        let t0 = Instant::now();

        assert_eq!(doc.reserve_inner(Flow::Emit, 6.0, t0), Reservation::Ready);
        doc.commit_inner(Flow::Emit, 6.0, t0);

        assert!(matches!(
            peer.reserve_inner(Flow::Emit, 6.0, t0),
            Reservation::After(_)
        ));
    }
}
