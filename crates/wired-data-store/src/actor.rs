use anyhow::{Context, Result};
use loro::LoroDoc;
use smol_str::SmolStr;
use xdid::{
    core::did::Did,
    methods::key::{Signer, p256::P256KeyPair},
};

use crate::{
    Genesis, RecordId, ValidatedView,
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
    pub async fn update_record<F>(&self, record_id: &RecordId, f: F) -> Result<()>
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

        // Export delta updates since the captured version.
        let ops = record
            .doc()
            .export(loro::ExportMode::updates(&from_version))
            .context("export updates")?;

        // Skip if no changes.
        if ops.is_empty() {
            return Ok(());
        }

        // Serialize version vector for envelope.
        let from_version_bytes = from_version.encode();

        // Create and sign envelope.
        let envelope = self.sign_envelope(record_id.clone(), ops, from_version_bytes)?;

        // Apply via validated view (verifies signature).
        self.view.apply_update(&envelope).await
    }

    /// Signs an envelope for the given record and operations.
    fn sign_envelope(
        &self,
        record_id: RecordId,
        ops: Vec<u8>,
        from_version: Vec<u8>,
    ) -> Result<Envelope> {
        // Build unsigned envelope for signable bytes.
        let mut envelope = Envelope {
            record_id,
            ops,
            from_version,
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
}
