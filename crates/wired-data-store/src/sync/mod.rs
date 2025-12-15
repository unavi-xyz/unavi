use iroh::EndpointId;
use xdid::core::did::Did;

use crate::RecordId;

mod connection;
mod messages;
mod protocol;
mod request;
mod session;
mod update;
mod wire;

pub use connection::ConnectionPool;
pub use messages::{SignatureWire, SyncMessage, SyncStatus};
pub use protocol::{ALPN, WiredSyncProtocol};

/// Event emitted when a record changes and needs sync.
#[derive(Clone, Debug)]
pub struct SyncEvent {
    pub record_id: RecordId,
    pub owner_did: Did,
    pub event_type: SyncEventType,
}

/// Type of sync event.
#[derive(Clone, Debug)]
pub enum SyncEventType {
    /// Record was created.
    Created,
    /// Record was updated.
    Updated,
    /// Record was deleted.
    Deleted,
}

/// Represents a sync peer identified by iroh [`EndpointId`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SyncPeer {
    pub endpoint_id: EndpointId,
}

impl SyncPeer {
    /// Creates a new sync peer from an iroh [`EndpointId`].
    #[must_use]
    pub const fn new(endpoint_id: EndpointId) -> Self {
        Self { endpoint_id }
    }

    /// Parses a sync peer from 32-byte [`EndpointId`].
    ///
    /// # Errors
    ///
    /// Returns error if bytes are not a valid Ed25519 public key.
    pub fn from_bytes(bytes: &[u8; 32]) -> anyhow::Result<Self> {
        Ok(Self {
            endpoint_id: EndpointId::from_bytes(bytes)?,
        })
    }

    /// Returns the [`EndpointId`] as 32 bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.endpoint_id.as_bytes()
    }
}
