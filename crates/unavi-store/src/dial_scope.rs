//! Which peers this endpoint may dial for document sync.
//!
//! The docs engine remembers past sync peers and dials all of them whenever a
//! sync starts. Those are mostly stale endpoints of earlier visitors, so each
//! costs a timeout and reveals this address to a peer nobody asked for.

use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};

use iroh::{
    EndpointAddr,
    EndpointId,
    endpoint::{
        AfterHandshakeOutcome,
        BeforeConnectOutcome,
        Connection,
        EndpointHooks,
    },
};
use n0_future::time::Instant;
use parking_lot::Mutex;

/// How long a peer named in a join stays dialable.
const NAMED_TTL: Duration = Duration::from_mins(10);

/// The most named peers kept at once.
const MAX_NAMED: usize = 4096;

/// Decides whether a docs dial is allowed. Cheap to clone; clones share state.
///
/// A docs dial is admitted when the remote holds a live connection with this
/// endpoint, or was named in a join within [`NAMED_TTL`].
#[derive(Debug, Clone)]
pub struct DialScope(Arc<Shared>);

#[derive(Debug)]
struct Shared {
    capacity: usize,
    state:    Mutex<State>,
}

#[derive(Debug, Default)]
struct State {
    named: HashMap<EndpointId, Instant>,
    live:  HashMap<EndpointId, usize>,
}

impl Default for DialScope {
    fn default() -> Self {
        Self::with_capacity(MAX_NAMED)
    }
}

impl DialScope {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn with_capacity(capacity: usize) -> Self {
        Self(Arc::new(Shared {
            capacity,
            state: Mutex::default(),
        }))
    }

    /// Allows dialing `peers` for the next [`NAMED_TTL`].
    pub fn name(&self, peers: &[EndpointId]) {
        self.name_at(peers, Instant::now());
    }

    #[must_use]
    pub fn admits(&self, peer: EndpointId) -> bool {
        self.admits_at(peer, Instant::now())
    }

    /// The hooks that enforce this scope on an endpoint's outgoing dials.
    #[must_use]
    pub fn hooks(&self) -> DialHooks {
        DialHooks(self.clone())
    }

    fn name_at(&self, peers: &[EndpointId], now: Instant) {
        let mut state = self.0.state.lock();
        for peer in peers {
            if !state.named.contains_key(peer) && state.named.len() >= self.0.capacity {
                state
                    .named
                    .retain(|_, named| now.saturating_duration_since(*named) < NAMED_TTL);
            }
            if !state.named.contains_key(peer) && state.named.len() >= self.0.capacity {
                let oldest = state
                    .named
                    .iter()
                    .min_by_key(|(_, named)| **named)
                    .map(|(id, _)| *id);
                if let Some(oldest) = oldest {
                    state.named.remove(&oldest);
                }
            }
            state.named.insert(*peer, now);
        }
    }

    fn admits_at(&self, peer: EndpointId, now: Instant) -> bool {
        let state = self.0.state.lock();
        state.live.contains_key(&peer)
            || state
                .named
                .get(&peer)
                .is_some_and(|named| now.saturating_duration_since(*named) < NAMED_TTL)
    }

    fn connection_opened(&self, peer: EndpointId) {
        *self.0.state.lock().live.entry(peer).or_default() += 1;
    }

    fn connection_closed(&self, peer: EndpointId) {
        let mut state = self.0.state.lock();
        if let Some(count) = state.live.get_mut(&peer) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                state.live.remove(&peer);
            }
        }
    }
}

/// Refuses docs dials outside the [`DialScope`] and counts live connections.
#[derive(Debug)]
pub struct DialHooks(DialScope);

impl EndpointHooks for DialHooks {
    fn before_connect<'a>(
        &'a self,
        remote_addr: &'a EndpointAddr,
        alpn: &'a [u8],
    ) -> impl Future<Output = BeforeConnectOutcome> + Send + 'a {
        let outcome = if alpn != iroh_docs::ALPN || self.0.admits(remote_addr.id) {
            BeforeConnectOutcome::Accept
        } else {
            BeforeConnectOutcome::Reject
        };
        std::future::ready(outcome)
    }

    fn after_handshake<'a>(
        &'a self,
        conn: &'a Connection,
    ) -> impl Future<Output = AfterHandshakeOutcome> + Send + 'a {
        let peer = conn.remote_id();
        // Registered now, while this call still holds the connection, so the
        // close is seen however soon it comes.
        let closed = conn.weak_handle().closed();
        self.0.connection_opened(peer);

        let scope = self.0.clone();
        n0_future::task::spawn(async move {
            closed.await;
            scope.connection_closed(peer);
        });

        std::future::ready(AfterHandshakeOutcome::accept())
    }
}

#[cfg(test)]
mod tests {
    use iroh::SecretKey;

    use super::*;

    fn peer(seed: u8) -> EndpointId {
        SecretKey::from_bytes(&[seed; 32]).public()
    }

    fn addr(seed: u8) -> EndpointAddr {
        EndpointAddr::new(peer(seed))
    }

    async fn outcome(scope: &DialScope, seed: u8, alpn: &[u8]) -> BeforeConnectOutcome {
        scope.hooks().before_connect(&addr(seed), alpn).await
    }

    #[test]
    fn a_named_peer_is_admitted_until_the_ttl_passes() {
        let scope = DialScope::new();
        let start = Instant::now();
        scope.name_at(&[peer(1)], start);

        assert!(scope.admits_at(peer(1), start));
        assert!(scope.admits_at(peer(1), start + NAMED_TTL - Duration::from_secs(1)));
        assert!(!scope.admits_at(peer(1), start + NAMED_TTL));
    }

    #[test]
    fn naming_a_peer_again_extends_its_window() {
        let scope = DialScope::new();
        let start = Instant::now();
        scope.name_at(&[peer(1)], start);
        let later = start + NAMED_TTL - Duration::from_secs(1);
        scope.name_at(&[peer(1)], later);

        assert!(scope.admits_at(peer(1), start + NAMED_TTL));
    }

    #[test]
    fn an_unnamed_peer_without_a_connection_is_refused() {
        let scope = DialScope::new();
        scope.name_at(&[peer(1)], Instant::now());

        assert!(!scope.admits_at(peer(2), Instant::now()));
    }

    #[test]
    fn a_live_connection_admits_an_unnamed_peer_until_the_last_one_closes() {
        let scope = DialScope::new();
        let now = Instant::now();
        scope.connection_opened(peer(1));
        scope.connection_opened(peer(1));

        assert!(scope.admits_at(peer(1), now));
        scope.connection_closed(peer(1));
        assert!(scope.admits_at(peer(1), now));
        scope.connection_closed(peer(1));
        assert!(!scope.admits_at(peer(1), now));
    }

    #[test]
    fn closing_a_connection_never_seen_changes_nothing() {
        let scope = DialScope::new();
        scope.connection_closed(peer(1));

        assert!(!scope.admits_at(peer(1), Instant::now()));
    }

    #[tokio::test]
    async fn other_alpns_are_always_admitted() {
        let scope = DialScope::new();

        let blobs = outcome(&scope, 1, iroh_blobs::ALPN).await;
        assert!(matches!(blobs, BeforeConnectOutcome::Accept));
        let gossip = outcome(&scope, 1, iroh_gossip::ALPN).await;
        assert!(matches!(gossip, BeforeConnectOutcome::Accept));
    }

    #[tokio::test]
    async fn the_hook_refuses_a_docs_dial_to_an_unnamed_peer() {
        let scope = DialScope::new();
        let refused = outcome(&scope, 1, iroh_docs::ALPN).await;
        assert!(matches!(refused, BeforeConnectOutcome::Reject));

        scope.name(&[peer(1)]);
        let admitted = outcome(&scope, 1, iroh_docs::ALPN).await;
        assert!(matches!(admitted, BeforeConnectOutcome::Accept));
    }

    #[test]
    fn the_named_set_stays_within_its_bound() {
        let scope = DialScope::with_capacity(3);
        let start = Instant::now();
        for seed in 1..=10 {
            scope.name_at(&[peer(seed)], start + Duration::from_secs(u64::from(seed)));
        }

        assert_eq!(scope.0.state.lock().named.len(), 3);
    }

    #[test]
    fn overflow_drops_expired_entries_before_live_ones() {
        let scope = DialScope::with_capacity(2);
        let start = Instant::now();
        scope.name_at(&[peer(1)], start);
        scope.name_at(&[peer(2)], start + NAMED_TTL);
        let now = start + NAMED_TTL + Duration::from_secs(1);
        scope.name_at(&[peer(3)], now);

        assert!(!scope.admits_at(peer(1), now));
        assert!(scope.admits_at(peer(2), now));
        assert!(scope.admits_at(peer(3), now));
    }

    #[test]
    fn overflow_with_nothing_expired_drops_the_oldest() {
        let scope = DialScope::with_capacity(2);
        let start = Instant::now();
        scope.name_at(&[peer(1)], start);
        scope.name_at(&[peer(2)], start + Duration::from_secs(1));
        let now = start + Duration::from_secs(2);
        scope.name_at(&[peer(3)], now);

        assert!(!scope.admits_at(peer(1), now));
        assert!(scope.admits_at(peer(2), now));
        assert!(scope.admits_at(peer(3), now));
    }
}
