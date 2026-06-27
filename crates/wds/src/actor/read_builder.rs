use std::time::Duration;

use anyhow::Context;
use blake3::Hash;
use iroh::EndpointAddr;
use loro::LoroDoc;
use tracing::debug;

use super::Actor;
use crate::{
    api::ReadRecord,
    error::ApiError,
};

/// Builder for reading records with optional sync fallbacks.
pub struct ReadBuilder {
    actor:        Actor,
    record_id:    Hash,
    ttl:          Duration,
    sync_sources: Vec<EndpointAddr>,
}

impl ReadBuilder {
    pub(super) const fn new(actor: Actor, record_id: Hash) -> Self {
        Self {
            actor,
            record_id,
            ttl: Duration::from_mins(30),
            sync_sources: Vec::new(),
        }
    }

    /// Pin TTL for the read record.
    #[must_use]
    pub const fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Add a remote endpoint to sync from if record not found locally.
    #[must_use]
    pub fn sync_from(mut self, endpoint: EndpointAddr) -> Self {
        self.sync_sources.push(endpoint);
        self
    }

    /// Execute the read.
    pub async fn send(self) -> anyhow::Result<LoroDoc> {
        let s = self.actor.authenticate().await.context("auth")?;

        debug!(record = ?self.record_id, "reading");

        // Pin record if not already pinned.
        if self.actor.get_record_pin(self.record_id).await?.is_none() {
            self.actor.pin_record(self.record_id, self.ttl).await?;
        }

        // Sync from each source before reading so the record's envelopes and
        // all its referenced blobs are present locally, even when a partial copy
        // of the record already exists.
        for remote in &self.sync_sources {
            debug!(remote = %remote.id, "attempting sync");
            if let Err(err) = self.actor.sync(self.record_id, remote.clone()).await {
                debug!(remote = %remote.id, ?err, "sync source did not have record");
            }
        }

        let result = self
            .actor
            .api_client
            .rpc(ReadRecord {
                s,
                record_id: self.record_id,
            })
            .await?;

        match result {
            Ok(bytes) => {
                let doc = LoroDoc::new();
                doc.import(&bytes)?;
                Ok(doc)
            }
            Err(ApiError::RecordNotFound) => anyhow::bail!("record not found"),
            Err(err) => Err(anyhow::anyhow!("read failed: {err}")),
        }
    }
}
