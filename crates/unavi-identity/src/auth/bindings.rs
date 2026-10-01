//! Which DID each connected peer has proven it controls.

use std::{
    collections::HashMap,
    pin::pin,
    sync::Arc,
    time::Duration,
};

use iroh::EndpointId;
use parking_lot::Mutex;
use tokio::sync::Notify;
use xdid::core::did::Did;

/// How long a binding outlives the peer's last connection. The dialer closes
/// its `wired/auth` connection before opening the one it proved itself for,
/// so a binding must survive the gap.
const GRACE: Duration = Duration::from_secs(30);

/// Maps a peer's endpoint id to the DID it has proven it controls.
///
/// Only a completed `wired/auth` handshake over that peer's own connection may
/// bind. A DID announced elsewhere is a claim anyone can make about anyone, so
/// an unbound peer is indistinguishable from an anonymous one.
///
/// A binding lasts while the peer holds a connection to this endpoint, plus
/// [`GRACE`], so the map never outgrows the peers actually connected.
#[derive(Debug, Default)]
pub struct Bindings {
    inner:   Mutex<Inner>,
    changed: Notify,
}

#[derive(Debug, Default)]
struct Inner {
    dids:        HashMap<EndpointId, Did>,
    connections: HashMap<EndpointId, usize>,
}

impl Bindings {
    /// Records `did` for `peer`. Only a verified handshake calls this.
    pub fn bind(&self, peer: EndpointId, did: Did) {
        self.inner.lock().dids.insert(peer, did);
        self.changed.notify_waiters();
    }

    pub fn unbind(&self, peer: EndpointId) {
        self.inner.lock().dids.remove(&peer);
    }

    #[must_use]
    pub fn did_of(&self, peer: EndpointId) -> Option<Did> {
        self.inner.lock().dids.get(&peer).cloned()
    }

    /// Every connected peer bound to `did`.
    #[must_use]
    pub fn peers_of(&self, did: &Did) -> Vec<EndpointId> {
        self.inner
            .lock()
            .dids
            .iter()
            .filter(|(_, bound)| *bound == did)
            .map(|(peer, _)| *peer)
            .collect()
    }

    #[must_use]
    pub fn is_bound(&self, peer: EndpointId) -> bool {
        self.inner.lock().dids.contains_key(&peer)
    }

    /// `peer`'s DID once it is bound, waiting up to `deadline` for a handshake
    /// still under way. `None` means the peer proved nothing in time, and is
    /// anonymous.
    ///
    /// A protocol that judges a peer by its DID awaits this before serving
    /// it, since a peer's connections race its `wired/auth` handshake.
    pub async fn bound(&self, peer: EndpointId, deadline: Duration) -> Option<Did> {
        let wait = async {
            loop {
                let mut changed = pin!(self.changed.notified());
                changed.as_mut().enable();
                if let Some(did) = self.did_of(peer) {
                    return did;
                }
                changed.await;
            }
        };
        n0_future::time::timeout(deadline, wait).await.ok()
    }

    /// How many peers are bound.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.lock().dids.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn connection_opened(&self, peer: EndpointId) {
        *self.inner.lock().connections.entry(peer).or_default() += 1;
    }

    /// Unbinds `peer` once it has held no connection for [`GRACE`].
    pub(crate) fn connection_closed(self: &Arc<Self>, peer: EndpointId) {
        {
            let mut inner = self.inner.lock();
            let Some(count) = inner.connections.get_mut(&peer) else {
                return;
            };
            *count = count.saturating_sub(1);
            if *count > 0 {
                return;
            }
            inner.connections.remove(&peer);
        }

        let bindings = Arc::downgrade(self);
        n0_future::task::spawn(async move {
            n0_future::time::sleep(GRACE).await;
            let Some(bindings) = bindings.upgrade() else {
                return;
            };
            let mut inner = bindings.inner.lock();
            if !inner.connections.contains_key(&peer) {
                inner.dids.remove(&peer);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use iroh::SecretKey;

    use super::*;

    fn did() -> Did {
        Did::from_str("did:web:example.com").expect("did")
    }

    #[tokio::test]
    async fn a_waiter_sees_a_later_binding() {
        let bindings = Arc::new(Bindings::default());
        let peer = SecretKey::generate().public();

        let waiter = {
            let bindings = Arc::clone(&bindings);
            tokio::spawn(async move { bindings.bound(peer, Duration::from_secs(5)).await })
        };
        tokio::task::yield_now().await;
        bindings.bind(peer, did());

        assert_eq!(waiter.await.expect("join"), Some(did()));
    }

    #[tokio::test]
    async fn a_peer_that_never_proves_is_anonymous() {
        let bindings = Bindings::default();
        let peer = SecretKey::generate().public();

        assert_eq!(bindings.bound(peer, Duration::from_millis(10)).await, None);
    }
}
