use anyhow::{anyhow, bail};
use xdid::resolver::DidResolver;

use crate::{DataStoreView, Envelope, crypto};

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

        // Apply ops to Loro doc.
        self.inner
            .apply_ops(&envelope.record_id, &envelope.ops)
            .await?;

        Ok(())
    }

    /// Get the inner view for read operations.
    #[must_use]
    pub const fn inner(&self) -> &DataStoreView {
        &self.inner
    }
}
