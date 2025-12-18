use sha2::{Digest, Sha256};
use xdid::core::did::Did;

mod actor;
mod blob;
mod crypto;
mod db;
mod envelope;
mod gc;
mod quota;
mod record;
mod snapshot;
mod store;
mod sync;
mod validated;
mod view;

pub use actor::Actor;
pub use blob::{Blob, BlobId, MAX_BLOB_SIZE};
pub use envelope::{Envelope, Signature};
pub use gc::GarbageCollectStats;
pub use quota::{DEFAULT_QUOTA_BYTES, QuotaExceeded, UserQuota};
pub use record::{Genesis, Pin, Record, RecordId};
pub use snapshot::{SNAPSHOT_BYTES_THRESHOLD, SNAPSHOT_OPS_THRESHOLD, SignedSnapshot};
pub use store::{DataStore, DataStoreBuilder};
pub use sync::{
    ALPN, ConnectionPool, SignatureWire, SyncEvent, SyncEventType, SyncMessage, SyncPeer,
    SyncStatus, WiredSyncProtocol,
};
pub use validated::ValidatedView;
pub use view::DataStoreView;

fn hash_did(did: &Did) -> String {
    let hash = Sha256::digest(did.to_string().as_bytes());
    format!("{hash:x}")[..16].to_string()
}
