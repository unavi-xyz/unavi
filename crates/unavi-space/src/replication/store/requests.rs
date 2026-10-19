//! Rate limit on state requests, so a peer cannot make this node build
//! snapshots in a loop.

use std::{
    collections::HashMap,
    time::Duration,
};

use iroh::EndpointId;
use n0_future::time::Instant;

use crate::membership::SpaceId;

/// Time to earn back one request.
pub const REQUEST_INTERVAL: Duration = Duration::from_secs(2);

/// Requests one peer may make for one space in a burst, which lets a space
/// reloaded a few times in quick succession ask each time.
pub const REQUEST_BURST: u32 = 3;

/// Most (peer, space) pairs tracked at once.
const MAX_TRACKED: usize = 1024;

/// A pair's allowance: `spent` requests, forgiven one per [`REQUEST_INTERVAL`]
/// since `at`.
struct Spent {
    spent: u32,
    at:    Instant,
}

impl Spent {
    /// What is still owed at `now`, after forgiving the elapsed intervals.
    fn owed(&self, now: Instant) -> u32 {
        let elapsed = now.saturating_duration_since(self.at);
        let forgiven = elapsed.as_millis() / REQUEST_INTERVAL.as_millis();
        self.spent
            .saturating_sub(u32::try_from(forgiven).unwrap_or(u32::MAX))
    }
}

#[derive(Default)]
pub(super) struct RequestLimiter {
    pairs: HashMap<(EndpointId, SpaceId), Spent>,
}

impl RequestLimiter {
    /// Spends one request of `peer`'s for `space`, answering whether it was
    /// allowed.
    pub(super) fn admit(&mut self, peer: EndpointId, space: SpaceId, now: Instant) -> bool {
        if self.pairs.len() >= MAX_TRACKED && !self.pairs.contains_key(&(peer, space)) {
            self.pairs.retain(|_, spent| spent.owed(now) > 0);
            if self.pairs.len() >= MAX_TRACKED {
                return false;
            }
        }
        let entry = self.pairs.entry((peer, space)).or_insert(Spent {
            spent: 0,
            at:    now,
        });
        let owed = entry.owed(now);
        if owed >= REQUEST_BURST {
            return false;
        }
        // Advance the clock by the whole intervals forgiven, keeping the part
        // interval already elapsed. With nothing owed it restarts at `now`.
        let at = if owed == 0 {
            now
        } else {
            entry.at + REQUEST_INTERVAL * (entry.spent - owed)
        };
        *entry = Spent {
            spent: owed + 1,
            at,
        };
        true
    }
}
