use blake3::Hash;
use bytes::Bytes;
use irpc::Client;
use xdid::{core::did::Did, methods::key::p256::P256KeyPair};

use crate::api::{ApiService, UploadBlob};

pub struct Actor {
    did: Did,
    signing_key: P256KeyPair,
    client: Client<ApiService>,
    authenticated: bool,
}

impl Actor {
    #[must_use]
    pub const fn new(did: Did, signing_key: P256KeyPair, client: Client<ApiService>) -> Self {
        Self {
            did,
            signing_key,
            client,
            authenticated: false,
        }
    }

    #[must_use]
    pub const fn did(&self) -> &Did {
        &self.did
    }

    /// Uplods bytes as to the WDS as a blob.
    /// Returns the blob hash.
    ///
    /// # Errors
    ///  
    /// Errors if the blob could not be uploaded, such as if the client disconnects
    /// or the storage quota is hit.
    pub async fn upload_blob(&self, bytes: Bytes) -> anyhow::Result<Hash> {
        let (tx, rx) = self.client.client_streaming(UploadBlob, 4).await?;

        tx.send(bytes).await?;

        let hash = rx
            .await?
            .map_err(|e| anyhow::anyhow!("upload failed: {e}"))?;

        Ok(hash)
    }
}
