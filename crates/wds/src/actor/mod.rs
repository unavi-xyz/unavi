use std::{sync::Arc, time::Duration};

use anyhow::Context;
use blake3::Hash;
use bytes::Bytes;
use iroh::EndpointId;
use irpc::Client;
use loro::LoroDoc;
use time::OffsetDateTime;
use tokio::sync::{Mutex, OnceCell};
use xdid::{core::did::Did, methods::key::p256::P256KeyPair};

use crate::{
    SessionToken,
    api::{ApiService, PinRecord, UploadBlob},
    auth::AuthService,
    record::{Record, acl::Acl, envelope::Envelope},
    signed_bytes::Signable,
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

const DEFAULT_PIN_TTL: Duration = Duration::from_hours(1);

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
    pub async fn create_record(&self, schemas: Option<Vec<Hash>>) -> anyhow::Result<Hash> {
        let s = self.authenticate().await.context("auth")?;

        let doc = LoroDoc::new();

        let mut record = Record::new(self.did.clone());

        for schema in schemas.unwrap_or_default() {
            record.add_schema(schema);
        }

        record.save(&doc)?;

        let mut acl = Acl::default();
        acl.manager.push(self.did.clone());
        acl.writer.push(self.did.clone());
        acl.save(&doc)?;

        let envelope = Envelope::all_updates(self.did.clone(), &doc)?;
        let _signed = envelope.sign(&self.signing_key)?;

        let id = record.id()?;

        self.api_client
            .rpc(PinRecord {
                s,
                id,
                expires: (OffsetDateTime::now_utc() + DEFAULT_PIN_TTL).unix_timestamp(),
            })
            .await?
            .map_err(|e| anyhow::anyhow!("record pin failed: {e}"))?;

        // TODO: sync?

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
