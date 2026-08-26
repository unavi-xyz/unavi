use unavi_quota::limits::Limits;

use crate::trust::Trust;

/// What a peer's content may consume: [`Limits::peer`] scaled by the peer
/// tier's share.
#[must_use]
pub fn for_trust(trust: Trust) -> Limits {
    let mut limits = Limits::peer();
    let share = match trust {
        // Not "a very small share": a blocked peer's content gets nothing, and
        // a zero-capacity bucket is refused on sight rather than waited on.
        Trust::Blocked => 0.0,
        Trust::Guest => 0.25,
        Trust::Known => 0.5,
        Trust::Trusted | Trust::Myself => return limits,
    };

    for cap in limits.stock.values_mut() {
        *cap = (*cap as f64 * share) as u64;
    }
    for limit in limits.flow.values_mut() {
        limit.capacity *= share;
        limit.refill_per_sec *= share;
    }
    limits
}

#[cfg(test)]
mod tests {
    use unavi_quota::{
        Flow,
        Quota,
        Reservation,
        Stock,
    };

    use super::*;

    #[test]
    fn a_blocked_peer_gets_nothing_and_is_refused_immediately() {
        let quota = Quota::root(for_trust(Trust::Blocked));

        assert_eq!(
            quota.reserve(Flow::CreatePrim, 1.0),
            Reservation::Never,
            "a zero bucket never fills, so waiting on it would be a lie"
        );
        assert!(quota.try_charge(Stock::Prims, 1).is_err());
    }

    #[test]
    fn the_rungs_are_ordered_by_what_they_may_consume() {
        let prims = |trust| {
            for_trust(trust)
                .stock
                .get(&Stock::Prims)
                .copied()
                .expect("peer limits cap prims")
        };

        assert!(prims(Trust::Blocked) < prims(Trust::Guest));
        assert!(prims(Trust::Guest) < prims(Trust::Known));
        assert!(prims(Trust::Known) < prims(Trust::Trusted));
        assert_eq!(prims(Trust::Trusted), prims(Trust::Myself));
    }

    #[test]
    fn a_guest_still_gets_a_workable_budget() {
        let quota = Quota::root(for_trust(Trust::Guest));
        assert_eq!(
            quota.reserve(Flow::CreatePrim, 100.0),
            Reservation::Ready,
            "a first-time visitor's prop must build without waiting"
        );
    }
}
