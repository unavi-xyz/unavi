use bincode::{Decode, Encode};

/// Protocol message types sent over iroh streams.
#[derive(Encode, Decode, Debug, Clone)]
pub enum SyncMessage {
    /// Initial handshake: "I am {`requester_did`}, I want to sync {`owner_did`}'s {`record_id`}".
    SyncRequest {
        requester_did: String,
        owner_did: String,
        record_id: String,
        /// Requester's current version vector (Loro `oplog_vv` bytes).
        version: Vec<u8>,
    },

    /// Response to [`SyncMessage::SyncRequest`].
    SyncResponse {
        status: SyncStatus,
        /// Loro ops to apply (if Syncing).
        ops: Vec<u8>,
        /// Signature over ops (from owner's delegated key).
        signature: Option<SignatureWire>,
    },

    /// Push a new update to connected peer.
    UpdatePush {
        record_id: String,
        owner_did: String,
        ops: Vec<u8>,
        from_version: Vec<u8>,
        signature: SignatureWire,
    },

    /// Acknowledge receipt of update.
    UpdateAck {
        record_id: String,
        owner_did: String,
    },
}

/// Response status for sync requests.
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
pub enum SyncStatus {
    /// Already in sync, no updates needed.
    InSync,
    /// Sending updates to bring requester up to date.
    Syncing,
    /// Record not found.
    NotFound,
    /// Access denied.
    AccessDenied,
    /// Requester identity verification failed.
    Unauthorized,
}

/// Wire format for cryptographic signature.
#[derive(Encode, Decode, Debug, Clone)]
pub struct SignatureWire {
    /// Algorithm identifier (e.g., "ES256", "`EdDSA`").
    pub alg: String,
    /// Raw signature bytes.
    pub bytes: Vec<u8>,
}
