use sha2::{Digest, Sha256};
use xdid::core::did::Did;

mod blob;
mod crypto;
mod db;
mod gc;
mod quota;
mod record;
mod store;
mod update;
mod validated;
mod view;

pub use blob::{Blob, BlobId, MAX_BLOB_SIZE};
pub use gc::GarbageCollectStats;
pub use quota::{DEFAULT_QUOTA_BYTES, QuotaExceeded, UserQuota};
pub use record::{Genesis, Pin, Record, RecordId};
pub use store::DataStore;
pub use update::{Signature, SignedUpdate};
pub use validated::ValidatedView;
pub use view::DataStoreView;

pub(crate) fn hash_did(did: &Did) -> String {
    let hash = Sha256::digest(did.to_string().as_bytes());
    format!("{hash:x}")[..16].to_string()
}
