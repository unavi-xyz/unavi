use std::time::Duration;

use anyhow::Context;
use blake3::Hash;
use iroh::EndpointAddr;
use loro::LoroDoc;
use tracing::{debug, warn};

use crate::api::{ApiError, ReadRecord};

use super::Actor;

/// Builder for reading records with optional sync fallbacks.
pub struct ReadBuilder {
    actor: Actor,
    record_id: Hash,
    ttl: Duration,
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

        // Try to read from local first.
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
                // TODO: Still sync local record with remote sources

                let doc = LoroDoc::new();
                doc.import(&bytes)?;
                return Ok(doc);
            }
            Err(ApiError::RecordNotFound) => {
                // Try sync below.
            }
            Err(e) => return Err(anyhow::anyhow!("read failed: {e}")),
        }

        // Check each sync source.
        for remote in self.sync_sources {
            let remote_id = remote.id;
            debug!(remote = %remote_id, "attempting sync");

            if let Err(e) = self.actor.sync(self.record_id, remote).await {
                warn!(remote = %remote_id, err = ?e, "sync failed");
                continue;
            }

            // Try reading again after sync.
            let result = self
                .actor
                .api_client
                .rpc(ReadRecord {
                    s,
                    record_id: self.record_id,
                })
                .await?;

            if let Ok(bytes) = result {
                let doc = LoroDoc::new();
                doc.import(&bytes)?;
                return Ok(doc);
            }
        }

        anyhow::bail!("record not found")
    }
}
