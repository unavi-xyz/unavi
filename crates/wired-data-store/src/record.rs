use cid::Cid;
use loro::LoroDoc;
use multihash::Multihash;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use smol_str::SmolStr;
use xdid::core::did::Did;

/// CID of the genesis block, uniquely identifies a record.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordId(pub Cid);

impl RecordId {
    #[must_use]
    pub fn as_str(&self) -> String {
        self.0.to_string()
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

    /// Computes the CID of this genesis block.
    ///
    /// # Panics
    ///
    /// Panics if SHA-256 digest cannot be wrapped in multihash (should never happen).
    #[must_use]
    pub fn cid(&self) -> Cid {
        // 0x12 = sha2-256, 0x55 = raw codec.
        let bytes = self.to_bytes();
        let digest = Sha256::digest(&bytes);
        let hash = Multihash::<64>::wrap(0x12, &digest).expect("sha256 multihash");
        Cid::new_v1(0x55, hash)
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
        let id = RecordId(genesis.cid());
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
