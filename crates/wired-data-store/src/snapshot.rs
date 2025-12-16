use crate::{RecordId, envelope::Signature};

/// Signed snapshot of a record's state at a specific version.
///
/// Snapshots consolidate Loro operations into a single exportable state,
/// enabling efficient sync by providing a known-good base that peers can
/// build upon without replaying all historical operations.
#[derive(Clone, Debug)]
pub struct SignedSnapshot {
    /// Target record CID.
    pub record_id: RecordId,
    /// Monotonically increasing snapshot number for this record.
    pub snapshot_num: u64,
    /// Version vector at snapshot time (encoded).
    pub version: Vec<u8>,
    /// Serialized Genesis for record reconstruction.
    pub genesis_bytes: Vec<u8>,
    /// Loro snapshot export (`ExportMode::snapshot`).
    pub loro_snapshot: Vec<u8>,
    /// Signature over canonical encoding.
    pub signature: Signature,
}

impl SignedSnapshot {
    /// Canonical bytes for signing/verification.
    ///
    /// # Panics
    ///
    /// Panics if any field length exceeds `u32::MAX`.
    #[must_use]
    pub fn signable_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend(self.record_id.as_str().as_bytes());
        buf.extend(&self.snapshot_num.to_be_bytes());
        buf.extend(
            &u32::try_from(self.version.len())
                .expect("version length exceeds u32::MAX")
                .to_be_bytes(),
        );
        buf.extend(&self.version);
        buf.extend(
            &u32::try_from(self.genesis_bytes.len())
                .expect("genesis_bytes length exceeds u32::MAX")
                .to_be_bytes(),
        );
        buf.extend(&self.genesis_bytes);
        buf.extend(
            &u32::try_from(self.loro_snapshot.len())
                .expect("loro_snapshot length exceeds u32::MAX")
                .to_be_bytes(),
        );
        buf.extend(&self.loro_snapshot);
        buf
    }
}

/// Thresholds for triggering automatic snapshots.
pub const SNAPSHOT_OPS_THRESHOLD: u64 = 1000;
pub const SNAPSHOT_BYTES_THRESHOLD: u64 = 8 * 1024 * 1024;
