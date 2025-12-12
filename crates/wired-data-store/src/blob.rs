use cid::Cid;
use multihash::Multihash;
use sha2::{Digest, Sha256};

/// CID identifying a blob (content hash).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlobId(pub Cid);

impl BlobId {
    /// Computes `BlobId` from content bytes.
    ///
    /// # Panics
    ///
    /// Panics if SHA-256 digest cannot be wrapped in multihash (should never happen).
    #[must_use]
    pub fn from_bytes(data: &[u8]) -> Self {
        let digest = Sha256::digest(data);
        let hash = Multihash::<64>::wrap(0x12, &digest).expect("sha256 multihash"); // 0x12 = sha2-256
        let cid = Cid::new_v1(0x55, hash); // 0x55 = raw codec
        Self(cid)
    }

    #[must_use]
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

/// Metadata about a stored blob.
pub struct Blob {
    pub id: BlobId,
    pub size: u64,
}
