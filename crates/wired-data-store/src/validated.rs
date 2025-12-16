use anyhow::{anyhow, bail};
use xdid::resolver::DidResolver;

use crate::{DataStoreView, Envelope, SNAPSHOT_BYTES_THRESHOLD, SNAPSHOT_OPS_THRESHOLD, crypto};

/// Wrapper providing signature verification for `DataStoreView`.
pub struct ValidatedView {
    inner: DataStoreView,
    resolver: DidResolver,
}

impl ValidatedView {
    /// Create a new validated view.
    ///
    /// # Errors
    ///
    /// Returns an error if the resolver fails to initialize.
    pub fn new(inner: DataStoreView) -> anyhow::Result<Self> {
        let resolver = DidResolver::new()?;
        Ok(Self { inner, resolver })
    }

    /// Apply signed update after verification.
    ///
    /// # Errors
    ///
    /// Returns an error if the update could not be validated or applied.
    pub async fn apply_update(&self, envelope: &Envelope) -> anyhow::Result<()> {
        // Resolve author's DID document.
        let document = self.resolver.resolve(&envelope.author).await?;

        // Extract JWK from verification method.
        let verification_method = document
            .assertion_method
            .as_ref()
            .and_then(|methods| {
                methods
                    .iter()
                    .find_map(|m| document.resolve_verification_method(m))
            })
            .ok_or_else(|| anyhow!("no assertion method"))?;

        let jwk = verification_method
            .public_key_jwk
            .as_ref()
            .ok_or_else(|| anyhow!("no public key"))?;

        // Verify signature.
        if !crypto::verify_jwk_signature(jwk, &envelope.signature.bytes, &envelope.signable_bytes())
        {
            bail!("invalid signature");
        }

        // Check authorization.
        let record = self
            .inner
            .get_record(&envelope.record_id)
            .await?
            .ok_or_else(|| anyhow!("record not found"))?;

        if record.genesis.creator != envelope.author {
            return Err(anyhow!("unauthorized: only creator can update"));
        }

        // Store the envelope for sync traceability.
        // Op ranges are computed from version vector diffs (empty for now, will be
        // computed when sync is implemented).
        let op_ranges: Vec<(u64, u64, u64)> = Vec::new();
        self.inner.store_envelope(envelope, &op_ranges).await?;

        // Apply ops to Loro doc.
        self.inner
            .apply_ops(&envelope.record_id, &envelope.ops)
            .await?;

        // Check if we should trigger a snapshot.
        let (ops_count, bytes_count, _snapshot_num) = self
            .inner
            .get_snapshot_counters(&envelope.record_id)
            .await?;

        let ops_threshold = i64::try_from(SNAPSHOT_OPS_THRESHOLD).unwrap_or(i64::MAX);
        let bytes_threshold = i64::try_from(SNAPSHOT_BYTES_THRESHOLD).unwrap_or(i64::MAX);

        if ops_count >= ops_threshold || bytes_count >= bytes_threshold {
            // Snapshot creation requires signing capability, which ValidatedView
            // doesn't have. Signal that a snapshot is needed via return value or
            // callback (for now, just log/note it - Actor will handle this).
            tracing::debug!(
                record_id = %envelope.record_id.as_str(),
                ops_count,
                bytes_count,
                "snapshot threshold reached"
            );
        }

        Ok(())
    }

    /// Get the inner view for read operations.
    #[must_use]
    pub const fn inner(&self) -> &DataStoreView {
        &self.inner
    }
}
