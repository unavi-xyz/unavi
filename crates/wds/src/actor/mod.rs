use std::{sync::Arc, time::Duration};

use anyhow::Context;
use blake3::Hash;
use bytes::Bytes;
use iroh::EndpointId;
use irpc::Client;
use loro::{LoroDoc, VersionVector};
use time::OffsetDateTime;
use tokio::sync::{Mutex, OnceCell};
use xdid::core::did::Did;

use crate::{
    Identity, SessionToken,
    api::{ApiService, PinBlob, PinRecord, UploadBlob, UploadEnvelope},
    auth::AuthService,
    record::envelope::Envelope,
    signed_bytes::{Signable, SignedBytes},
};

mod auth;
mod into_actor;
mod record_builder;

pub use record_builder::RecordResult;

/// Authenticated agent for WDS operations.
///
/// An actor targets a specific WDS host and performs authenticated operations.
/// The same [`Identity`] can be shared across multiple actors targeting
/// different hosts.
#[derive(Clone)]
pub struct Actor {
    identity: Arc<Identity>,
    host: EndpointId,
    api_client: Client<ApiService>,
    auth_client: Client<AuthService>,
    session: Arc<Mutex<OnceCell<SessionToken>>>,
}

impl Actor {
    pub(crate) fn new(
        identity: Arc<Identity>,
        host: EndpointId,
        api_client: Client<ApiService>,
        auth_client: Client<AuthService>,
    ) -> Self {
        Self {
            identity,
            host,
            api_client,
            auth_client,
            session: Arc::new(Mutex::new(OnceCell::default())),
        }
    }

    #[must_use]
    pub fn did(&self) -> &Did {
        self.identity.did()
    }

    #[must_use]
    pub const fn identity(&self) -> &Arc<Identity> {
        &self.identity
    }

    #[must_use]
    pub const fn host(&self) -> &EndpointId {
        &self.host
    }

    #[must_use]
    pub fn signing_key(&self) -> &xdid::methods::key::p256::P256KeyPair {
        self.identity.signing_key()
    }

    /// Creates a new record, returning the record ID.
    ///
    /// # Errors
    ///
    /// Errors if the record could not be created, such as if the client disconnects
    /// or the storage quota is hit.
    #[must_use]
    pub fn create_record(&self) -> record_builder::RecordBuilder {
        record_builder::RecordBuilder::new(self.clone())
    }

    /// Uploads a signed envelope to a record.
    ///
    /// # Errors
    ///
    /// Errors if the envelope could not be uploaded, such as if the client
    /// disconnects, the storage quota is hit, or the envelope is invalid.
    pub async fn upload_envelope(
        &self,
        record_id: Hash,
        envelope: SignedBytes<Envelope>,
    ) -> anyhow::Result<()> {
        let s = self.authenticate().await.context("auth")?;

        let env_bytes = postcard::to_stdvec(&envelope)?;

        let (tx, rx) = self
            .api_client
            .client_streaming(UploadEnvelope { s, record_id }, 4)
            .await
            .context("init upload envelope")?;

        tx.send(env_bytes.into()).await.context("send envelope")?;
        drop(tx);

        rx.await?
            .map_err(|e| anyhow::anyhow!("upload envelope failed: {e}"))?;

        Ok(())
    }

    /// Updates a record by creating and uploading an envelope from the doc.
    ///
    /// # Errors
    ///
    /// Errors if the envelope could not be created or uploaded.
    pub async fn update_record(
        &self,
        record_id: Hash,
        doc: &LoroDoc,
        from: VersionVector,
    ) -> anyhow::Result<()> {
        let envelope = Envelope::updates(self.identity.did().clone(), doc, from)?;
        let signed = envelope.sign(self.identity.signing_key())?;
        self.upload_envelope(record_id, signed).await
    }

    /// Uploads bytes to the WDS as a blob.
    /// Returns the blob hash.
    ///
    /// # Errors
    ///
    /// Errors if the blob could not be uploaded, such as if the client disconnects
    /// or the storage quota is hit.
    pub async fn upload_blob(&self, bytes: Bytes) -> anyhow::Result<Hash> {
        let s = self.authenticate().await.context("auth")?;

        let (tx, rx) = self
            .api_client
            .client_streaming(UploadBlob { s }, 4)
            .await
            .context("init upload blob")?;

        tx.send(bytes).await.context("send bytes")?;
        drop(tx);

        let hash = rx
            .await?
            .map_err(|e| anyhow::anyhow!("upload failed: {e}"))?;

        Ok(hash)
    }

    /// Pins a record at this actor's host for the given duration.
    ///
    /// # Errors
    ///
    /// Errors if the pin request fails.
    pub async fn pin_record(&self, id: Hash, ttl: Duration) -> anyhow::Result<()> {
        let s = self.authenticate().await.context("auth")?;
        let expires = (OffsetDateTime::now_utc() + ttl).unix_timestamp();

        self.api_client
            .rpc(PinRecord { s, id, expires })
            .await?
            .map_err(|e| anyhow::anyhow!("pin record failed: {e}"))?;

        Ok(())
    }

    /// Pins a blob at this actor's host for the given duration.
    ///
    /// # Errors
    ///
    /// Errors if the pin request fails.
    pub async fn pin_blob(&self, hash: Hash, ttl: Duration) -> anyhow::Result<()> {
        let s = self.authenticate().await.context("auth")?;
        let expires = (OffsetDateTime::now_utc() + ttl).unix_timestamp();

        self.api_client
            .rpc(PinBlob { s, hash, expires })
            .await?
            .map_err(|e| anyhow::anyhow!("pin blob failed: {e}"))?;

        Ok(())
    }
}
