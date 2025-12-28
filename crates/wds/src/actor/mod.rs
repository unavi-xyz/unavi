use std::sync::Arc;

use anyhow::Context;
use blake3::Hash;
use bytes::Bytes;
use iroh::EndpointId;
use irpc::Client;
use loro::{LoroDoc, VersionVector};
use tokio::sync::{Mutex, OnceCell};
use xdid::{core::did::Did, methods::key::p256::P256KeyPair};

use crate::{
    SessionToken,
    api::{ApiService, UploadBlob, UploadEnvelope},
    auth::AuthService,
    record::envelope::Envelope,
    signed_bytes::{Signable, SignedBytes},
};

mod auth;
mod record_builder;

#[derive(Clone)]
pub struct Actor {
    did: Did,
    signing_key: P256KeyPair,
    host: EndpointId,
    api_client: Client<ApiService>,
    auth_client: Client<AuthService>,
    session: Arc<Mutex<OnceCell<SessionToken>>>,
}

impl Actor {
    pub(crate) fn new(
        did: Did,
        signing_key: P256KeyPair,
        host: EndpointId,
        api_client: Client<ApiService>,
        auth_client: Client<AuthService>,
    ) -> Self {
        Self {
            did,
            signing_key,
            host,
            api_client,
            auth_client,
            session: Arc::new(Mutex::new(OnceCell::default())),
        }
    }

    #[must_use]
    pub const fn did(&self) -> &Did {
        &self.did
    }

    #[must_use]
    pub const fn signing_key(&self) -> &P256KeyPair {
        &self.signing_key
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
        let envelope = Envelope::updates(self.did.clone(), doc, from)?;
        let signed = envelope.sign(&self.signing_key)?;
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
}
