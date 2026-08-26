use std::{
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use blake3::Hash;
use bytes::Bytes;
use iroh::EndpointAddr;
use iroh_docs::NamespaceId;
use irpc::Client;
use time::OffsetDateTime;
use tokio::sync::{
    Mutex,
    OnceCell,
};

use crate::{
    SessionToken,
    auth::AuthService,
    control::{
        BlobExists,
        ControlService,
        GetQuota,
        HostDoc,
        PinBlob,
        QuotaInfo,
        UnhostDoc,
        UploadBlob,
    },
    identity::Identity,
};

mod auth;

/// Authenticated actor for WDS control-plane operations.
///
/// Doc reads/writes go directly over iroh-docs; the actor carries only
/// hosting, pinning, uploads, and quota.
#[derive(Clone)]
pub struct Actor {
    identity:       Arc<Identity>,
    host:           EndpointAddr,
    control_client: Client<ControlService>,
    auth_client:    Client<AuthService>,
    session:        Arc<Mutex<OnceCell<SessionToken>>>,
}

impl Actor {
    pub(crate) fn new(
        identity: Arc<Identity>,
        host: EndpointAddr,
        control_client: Client<ControlService>,
        auth_client: Client<AuthService>,
    ) -> Self {
        Self {
            identity,
            host,
            control_client,
            auth_client,
            session: Arc::new(Mutex::new(OnceCell::default())),
        }
    }

    #[must_use]
    pub const fn identity(&self) -> &Arc<Identity> {
        &self.identity
    }

    #[must_use]
    pub const fn host(&self) -> &EndpointAddr {
        &self.host
    }

    /// Uploads bytes to the WDS as a blob, returning the blob hash.
    pub async fn upload_blob(&self, bytes: Bytes) -> anyhow::Result<Hash> {
        let s = self.authenticate().await.context("auth")?;

        let (tx, rx) = self
            .control_client
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

    /// Pins a blob at this actor's host for the given duration.
    pub async fn pin_blob(&self, hash: Hash, ttl: Duration) -> anyhow::Result<()> {
        let s = self.authenticate().await.context("auth")?;
        let expires = (OffsetDateTime::now_utc() + ttl).unix_timestamp();

        self.control_client
            .rpc(PinBlob { s, hash, expires })
            .await?
            .map_err(|e| anyhow::anyhow!("pin blob failed: {e}"))?;

        Ok(())
    }

    /// Checks if a blob exists at this actor's host.
    pub async fn blob_exists(&self, hash: Hash) -> anyhow::Result<bool> {
        let s = self.authenticate().await.context("auth")?;

        let exists = self
            .control_client
            .rpc(BlobExists { s, hash })
            .await?
            .map_err(|e| anyhow::anyhow!("blob exists check failed: {e}"))?;

        Ok(exists)
    }

    /// Asks this actor's host to replicate a doc, charged to the actor's quota.
    pub async fn host_doc(&self, ns: NamespaceId) -> anyhow::Result<()> {
        let s = self.authenticate().await.context("auth")?;

        self.control_client
            .rpc(HostDoc { s, ns })
            .await?
            .map_err(|e| anyhow::anyhow!("host doc failed: {e}"))?;

        Ok(())
    }

    /// Asks this actor's host to stop replicating a doc.
    pub async fn unhost_doc(&self, ns: NamespaceId) -> anyhow::Result<()> {
        let s = self.authenticate().await.context("auth")?;

        self.control_client
            .rpc(UnhostDoc { s, ns })
            .await?
            .map_err(|e| anyhow::anyhow!("unhost doc failed: {e}"))?;

        Ok(())
    }

    /// Reports the actor's quota usage at this host.
    pub async fn get_quota(&self) -> anyhow::Result<QuotaInfo> {
        let s = self.authenticate().await.context("auth")?;

        let info = self
            .control_client
            .rpc(GetQuota { s })
            .await?
            .map_err(|e| anyhow::anyhow!("get quota failed: {e}"))?;

        Ok(info)
    }
}
