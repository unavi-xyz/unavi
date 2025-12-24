use std::sync::Arc;

use anyhow::Context;
use blake3::Hash;
use bytes::Bytes;
use iroh::EndpointId;
use irpc::Client;
use loro::LoroDoc;
use tokio::sync::{Mutex, OnceCell};
use xdid::{core::did::Did, methods::key::p256::P256KeyPair};

use crate::{
    SessionToken,
    api::{ApiService, CreateRecord, UploadBlob},
    auth::AuthService,
};

mod auth;

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

    /// Creates a new record, returning the record ID.
    ///
    /// # Errors
    ///  
    /// Errors if the record could not be created, such as if the client disconnects
    /// or the storage quota is hit.
    pub async fn create_record(&self, schema: Option<String>) -> anyhow::Result<Hash> {
        let s = self.authenticate().await.context("auth")?;

        let doc = LoroDoc::new();

        let id = self
            .api_client
            .rpc(CreateRecord { s, schema })
            .await?
            .map_err(|e| anyhow::anyhow!("creation failed: {e}"))?;

        Ok(id)
    }

    /// Uplods bytes as to the WDS as a blob.
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
