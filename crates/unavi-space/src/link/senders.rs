//! Per-peer outbound sender entities, which systems feed and connection tasks
//! drain.

use std::time::Duration;

use bevy::prelude::*;
use iroh::EndpointId;

/// Marks an entity feeding one peer's outbound stream. Despawned with the
/// connection.
#[derive(Component)]
#[require(SendInterval, LastSent)]
pub struct PeerSender(pub EndpointId);

/// How often a sender sends.
#[derive(Component)]
pub struct SendInterval(pub Duration);

impl Default for SendInterval {
    fn default() -> Self {
        Self(Duration::from_millis(50))
    }
}

/// When a sender last sent, in app time.
#[derive(Component, Default)]
pub struct LastSent(pub Duration);

impl LastSent {
    /// Whether `interval` has passed since the last send, stamping `now` if
    /// so.
    pub fn take_due(&mut self, interval: Duration, now: Duration) -> bool {
        if self.0 + interval > now {
            return false;
        }
        self.0 = now;
        true
    }
}
