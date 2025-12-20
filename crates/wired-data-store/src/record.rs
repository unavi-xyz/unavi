use std::fmt::Display;
use std::str::FromStr;

use iroh_blobs::Hash;
use loro::LoroDoc;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use xdid::core::did::Did;

/// Blake3 hash of the genesis block, uniquely identifies a record.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordId(pub Hash);

impl Display for RecordId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_hex())
    }
}

impl RecordId {
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

impl FromStr for RecordId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hash = Hash::from_str(s)?;
        Ok(Self(hash))
    }
}

/// Immutable genesis block of a record.
#[derive(Clone, Debug)]
pub struct Genesis {
    /// DID of the creator.
    pub creator: Did,
    /// Unix timestamp (seconds).
    pub created: u64,
    /// Random nonce for uniqueness.
    pub nonce: [u8; 16],
    /// Schema identifier.
    pub schema: Option<SmolStr>,
}

impl Genesis {
    /// Creates a new genesis block.
    ///
    /// # Panics
    ///
    /// Panics if random number generator fails or system time is before UNIX epoch.
    #[must_use]
    pub fn new(creator: Did) -> Self {
        let mut nonce = [0u8; 16];
        getrandom::fill(&mut nonce).expect("random nonce");

        Self {
            creator,
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time")
                .as_secs(),
            nonce,
            schema: None,
        }
    }

    /// Sets the schema for this genesis block.
    #[must_use]
    pub fn with_schema(mut self, schema: impl Into<SmolStr>) -> Self {
        self.schema = Some(schema.into());
        self
    }

    /// Computes the blake3 hash of this genesis block.
    #[must_use]
    pub fn hash(&self) -> Hash {
        Hash::new(self.to_bytes())
    }

    fn to_bytes(&self) -> Vec<u8> {
        // Deterministic serialization.
        let mut buf = Vec::new();
        buf.extend_from_slice(self.creator.to_string().as_bytes());
        buf.extend_from_slice(&self.created.to_be_bytes());
        buf.extend_from_slice(&self.nonce);
        if let Some(schema) = &self.schema {
            buf.extend_from_slice(schema.as_bytes());
        }
        buf
    }
}

/// A Record wrapping a Loro document.
pub struct Record {
    pub id: RecordId,
    pub genesis: Genesis,
    doc: LoroDoc,
}

impl Record {
    /// Creates a new record with the given genesis data.
    #[must_use]
    pub fn new(genesis: Genesis) -> Self {
        let id = RecordId(genesis.hash());
        let doc = LoroDoc::new();

        Self { id, genesis, doc }
    }

    /// Exports the full Loro document snapshot.
    ///
    /// # Errors
    ///
    /// Returns error if Loro export fails.
    pub fn export_snapshot(&self) -> anyhow::Result<Vec<u8>> {
        Ok(self.doc.export(loro::ExportMode::Snapshot)?)
    }

    /// Imports a snapshot into this document.
    ///
    /// # Errors
    ///
    /// Returns error if snapshot is invalid or cannot be imported.
    pub fn import_snapshot(&mut self, snapshot: &[u8]) -> anyhow::Result<()> {
        self.doc.import(snapshot)?;
        Ok(())
    }

    /// Access the underlying Loro document.
    #[must_use]
    pub const fn doc(&self) -> &LoroDoc {
        &self.doc
    }

    /// Mutably access the underlying Loro document.
    pub const fn doc_mut(&mut self) -> &mut LoroDoc {
        &mut self.doc
    }
}

/// A pin represents an explicit subscription to a record.
pub struct Pin {
    pub record_id: RecordId,
    pub created: u64,
    pub expires: Option<u64>,
}
