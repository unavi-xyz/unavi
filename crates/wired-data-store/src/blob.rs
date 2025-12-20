use std::fmt::Display;
use std::str::FromStr;

use iroh_blobs::Hash;

/// Maximum blob size: 512 MB.
pub const MAX_BLOB_SIZE: usize = 512 * 1024 * 1024;

/// Blake3 hash identifying a blob (content hash).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlobId(pub Hash);

impl BlobId {
    /// Computes `BlobId` from content bytes using blake3.
    #[must_use]
    pub fn from_bytes(data: &[u8]) -> Self {
        Self(Hash::new(data))
    }

    /// Returns the hex string representation.
    #[must_use]
    pub fn as_str(&self) -> String {
        self.0.to_hex()
    }

    /// Returns the underlying iroh Hash.
    #[must_use]
    pub const fn hash(&self) -> &Hash {
        &self.0
    }
}

impl Display for BlobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_hex())
    }
}

impl FromStr for BlobId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hash = Hash::from_str(s)?;
        Ok(Self(hash))
    }
}

/// Metadata about a stored blob.
pub struct Blob {
    pub id: BlobId,
    pub size: u64,
}
