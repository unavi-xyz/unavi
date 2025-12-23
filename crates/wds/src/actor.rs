use std::sync::Arc;

use anyhow::{Context, bail};
use blake3::Hash;
use bytes::Bytes;
use iroh::EndpointId;
use irpc::Client;
use tokio::sync::{Mutex, OnceCell};
use tracing::debug;
use xdid::{core::did::Did, methods::key::p256::P256KeyPair};

use crate::{
    SessionToken,
    api::{ApiService, UploadBlob},
    auth::{AnswerChallenge, AuthService, Challenge, RequestChallenge},
    signed_bytes::SignedBytes,
};

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
    #[must_use]
    pub fn new(
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

    async fn get_session_token(&self) -> anyhow::Result<SessionToken> {
        let session = self.session.lock().await;

        // If not authed, hold the lock while we authenticate.
        if let Some(s) = session.get().copied() {
            return Ok(s);
        }

        debug!("authenticating");

        let nonce = self
            .auth_client
            .rpc(RequestChallenge(self.did.clone()))
            .await
            .context("request challenge")?;

        let challenge = Challenge {
            did: self.did.clone(),
            host: self.host,
            nonce,
        };

        let signed = SignedBytes::sign(&challenge, &self.signing_key).context("sign challenge")?;

        let Some(s) = self
            .auth_client
            .rpc(AnswerChallenge(signed))
            .await
            .context("answer challenge rpc")?
        else {
            bail!("failed to authenticate")
        };

        session.set(s)?;
        drop(session);

        debug!("successfully authenticated");

        Ok(s)
    }

    /// Uplods bytes as to the WDS as a blob.
    /// Returns the blob hash.
    ///
    /// # Errors
    ///  
    /// Errors if the blob could not be uploaded, such as if the client disconnects
    /// or the storage quota is hit.
    pub async fn upload_blob(&self, bytes: Bytes) -> anyhow::Result<Hash> {
        let s = self.get_session_token().await.context("auth")?;

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
