use std::sync::Arc;

use anyhow::{Context, bail};
use blake3::Hash;
use bytes::Bytes;
use irpc::Client;
use tokio::sync::Mutex;
use tracing::debug;
use xdid::{core::did::Did, methods::key::p256::P256KeyPair};

use crate::{
    api::{ApiService, UploadBlob},
    auth::{AnswerChallenge, AuthService, Challenge, RequestChallenge},
    signed_bytes::SignedBytes,
};

#[derive(Clone)]
pub struct Actor {
    did: Did,
    signing_key: P256KeyPair,
    api_client: Client<ApiService>,
    auth_client: Client<AuthService>,
    authenticated: Arc<Mutex<bool>>,
}

impl Actor {
    #[must_use]
    pub fn new(
        did: Did,
        signing_key: P256KeyPair,
        api_client: Client<ApiService>,
        auth_client: Client<AuthService>,
    ) -> Self {
        Self {
            did,
            signing_key,
            api_client,
            auth_client,
            authenticated: Arc::new(Mutex::new(false)),
        }
    }

    #[must_use]
    pub const fn did(&self) -> &Did {
        &self.did
    }

    async fn ensure_authenticated(&self) -> anyhow::Result<()> {
        let mut authed = self.authenticated.lock().await;

        // If not authed, hold the lock while we authenticate.
        if *authed {
            return Ok(());
        }

        debug!("authenticating");

        let nonce = self
            .auth_client
            .rpc(RequestChallenge(self.did.clone()))
            .await
            .context("request challenge")?;

        let challenge = Challenge {
            did: self.did.clone(),
            nonce,
        };

        let signed = SignedBytes::sign(&challenge, &self.signing_key).context("sign challenge")?;

        let success = self
            .auth_client
            .rpc(AnswerChallenge(signed))
            .await
            .context("answer challenge rpc")?;

        if !success {
            bail!("failed to authenticate")
        }

        *authed = true;
        drop(authed);

        debug!("successfully authenticated");

        Ok(())
    }

    /// Uplods bytes as to the WDS as a blob.
    /// Returns the blob hash.
    ///
    /// # Errors
    ///  
    /// Errors if the blob could not be uploaded, such as if the client disconnects
    /// or the storage quota is hit.
    pub async fn upload_blob(&self, bytes: Bytes) -> anyhow::Result<Hash> {
        self.ensure_authenticated().await.context("auth")?;

        let (tx, rx) = self
            .api_client
            .client_streaming(UploadBlob, 4)
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
