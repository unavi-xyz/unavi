use anyhow::{Context, Result};
use loro::LoroDoc;
use smol_str::SmolStr;
use xdid::{
    core::did::Did,
    methods::key::{Signer, p256::P256KeyPair},
};

use crate::{
    Genesis, RecordId, SignedSnapshot, ValidatedView,
    envelope::{Envelope, Signature},
};

/// Algorithm identifier for P-256 ECDSA signatures.
const ALG_ES256: &str = "ES256";

/// An actor represents a single identity with signing capability.
pub struct Actor {
    did: Did,
    signing_key: P256KeyPair,
    view: ValidatedView,
}

impl Actor {
    /// Creates a new actor with the given identity and signing key.
    ///
    /// The DID should match the public key of the signing key (typically
    /// derived via `signing_key.public().to_did()`).
    #[must_use]
    pub const fn new(did: Did, signing_key: P256KeyPair, view: ValidatedView) -> Self {
        Self {
            did,
            signing_key,
            view,
        }
    }

    /// Returns the actor's DID.
    #[must_use]
    pub const fn did(&self) -> &Did {
        &self.did
    }

    /// Returns a reference to the underlying validated view.
    #[must_use]
    pub const fn view(&self) -> &ValidatedView {
        &self.view
    }

    /// Creates a new record with the actor as creator.
    ///
    /// # Errors
    ///
    /// Returns an error if the record could not be created.
    pub async fn create_record(&self, schema: Option<&str>) -> Result<RecordId> {
        let mut genesis = Genesis::new(self.did.clone());
        if let Some(s) = schema {
            genesis = genesis.with_schema(s);
        }
        self.view.inner().create_record(genesis).await
    }

    /// Updates a record by applying modifications via a callback.
    ///
    /// The callback receives a mutable reference to the Loro document,
    /// allowing modifications. After the callback completes, the changes
    /// are exported, signed, and applied via the validated view.
    ///
    /// # Errors
    ///
    /// Returns an error if the record could not be loaded, modified, or updated.
    ///
    /// # Example
    ///
    /// ```ignore
    /// actor.update_record(&record_id, |doc| {
    ///     let map = doc.get_map("space");
    ///     map.insert("name", "My Space")?;
    ///     Ok(())
    /// }).await?;
    /// ```
    pub async fn update_record<F>(&self, record_id: RecordId, f: F) -> Result<()>
    where
        F: FnOnce(&mut LoroDoc) -> Result<()>,
    {
        // Load current record state.
        let mut record = self
            .view
            .inner()
            .get_record(record_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("record not found"))?;

        // Capture version before modifications.
        let from_version = record.doc().oplog_vv();

        // Apply user modifications.
        f(record.doc_mut())?;

        // Capture version after modifications.
        let to_version = record.doc().oplog_vv();

        // Export delta updates since the captured version.
        let ops = record
            .doc()
            .export(loro::ExportMode::updates(&from_version))
            .context("export updates")?;

        // Skip if no changes.
        if ops.is_empty() {
            return Ok(());
        }

        // Serialize version vectors for envelope.
        let from_version_bytes = from_version.encode();
        let to_version_bytes = to_version.encode();

        // Create and sign envelope.
        let envelope =
            self.sign_envelope(record_id.clone(), ops, from_version_bytes, to_version_bytes)?;

        // Apply via validated view (verifies signature).
        self.view.apply_update(&envelope).await
    }

    /// Creates a signed snapshot of the record's current state.
    ///
    /// Snapshots consolidate Loro operations into a single exportable state,
    /// enabling efficient sync by providing a known-good base.
    ///
    /// After creating a snapshot, old envelopes are garbage collected using
    /// lagged deletion (keeping snapshot N-1 and its envelopes).
    ///
    /// # Errors
    ///
    /// Returns an error if the record cannot be loaded, snapshot cannot be
    /// created, or storage fails.
    pub async fn create_snapshot(&self, record_id: RecordId) -> Result<SignedSnapshot> {
        // Load current record state.
        let record = self
            .view
            .inner()
            .get_record(record_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("record not found"))?;

        // Get current snapshot number and increment.
        let (_ops_count, _bytes_count, current_num) =
            self.view.inner().get_snapshot_counters(record_id).await?;
        let snapshot_num = u64::try_from(current_num + 1).context("snapshot_num overflow")?;

        // Export Loro snapshot.
        let loro_snapshot = record
            .doc()
            .export(loro::ExportMode::Snapshot)
            .context("export loro snapshot")?;

        // Capture current version.
        let version = record.doc().oplog_vv().encode();

        // Serialize genesis (simple format: creator DID + created timestamp + nonce + schema).
        let genesis_bytes = serialize_genesis(&record.genesis);

        // Build and sign snapshot.
        let snapshot = self.sign_snapshot(
            record_id.clone(),
            snapshot_num,
            version,
            genesis_bytes,
            loro_snapshot,
        )?;

        // Store snapshot (also resets counters).
        self.view.inner().store_snapshot(&snapshot).await?;

        // GC old envelopes using lagged deletion.
        let deleted = self.view.inner().gc_old_envelopes(record_id).await?;
        if deleted > 0 {
            tracing::debug!(
                record_id = %record_id.as_str(),
                deleted,
                "garbage collected old envelopes"
            );
        }

        Ok(snapshot)
    }

    /// Signs an envelope for the given record and operations.
    fn sign_envelope(
        &self,
        record_id: RecordId,
        ops: Vec<u8>,
        from_version: Vec<u8>,
        to_version: Vec<u8>,
    ) -> Result<Envelope> {
        // Build unsigned envelope for signable bytes.
        let mut envelope = Envelope {
            record_id,
            ops,
            from_version,
            to_version,
            author: self.did.clone(),
            signature: Signature {
                alg: SmolStr::new(ALG_ES256),
                bytes: Vec::new(),
            },
        };

        // Sign canonical bytes.
        let signable = envelope.signable_bytes();
        let signature_bytes = self.signing_key.sign(&signable).context("sign envelope")?;

        envelope.signature.bytes = signature_bytes;
        Ok(envelope)
    }

    /// Signs a snapshot for the given record.
    fn sign_snapshot(
        &self,
        record_id: RecordId,
        snapshot_num: u64,
        version: Vec<u8>,
        genesis_bytes: Vec<u8>,
        loro_snapshot: Vec<u8>,
    ) -> Result<SignedSnapshot> {
        // Build unsigned snapshot for signable bytes.
        let mut snapshot = SignedSnapshot {
            record_id,
            snapshot_num,
            version,
            genesis_bytes,
            loro_snapshot,
            signature: Signature {
                alg: SmolStr::new(ALG_ES256),
                bytes: Vec::new(),
            },
        };

        // Sign canonical bytes.
        let signable = snapshot.signable_bytes();
        let signature_bytes = self.signing_key.sign(&signable).context("sign snapshot")?;

        snapshot.signature.bytes = signature_bytes;
        Ok(snapshot)
    }
}

/// Serializes a Genesis block into a canonical byte format.
///
/// Format: creator DID (length-prefixed) + created (u64 BE) + nonce (16 bytes) + schema (length-prefixed, 0 if None).
fn serialize_genesis(genesis: &Genesis) -> Vec<u8> {
    let mut buf = Vec::new();
    let creator_bytes = genesis.creator.to_string();
    buf.extend(
        &u32::try_from(creator_bytes.len())
            .expect("creator length exceeds u32::MAX")
            .to_be_bytes(),
    );
    buf.extend(creator_bytes.as_bytes());
    buf.extend(&genesis.created.to_be_bytes());
    buf.extend(&genesis.nonce);
    if let Some(ref schema) = genesis.schema {
        buf.extend(
            &u32::try_from(schema.len())
                .expect("schema length exceeds u32::MAX")
                .to_be_bytes(),
        );
        buf.extend(schema.as_bytes());
    } else {
        buf.extend(&0u32.to_be_bytes());
    }
    buf
}
