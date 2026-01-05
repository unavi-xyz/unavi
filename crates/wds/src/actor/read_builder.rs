use anyhow::Context;
use blake3::Hash;
use iroh::EndpointAddr;
use loro::LoroDoc;

use crate::api::ReadRecord;

use super::Actor;

/// Builder for reading records with optional sync fallbacks.
pub struct ReadBuilder {
    actor: Actor,
    record_id: Hash,
    sync_sources: Vec<EndpointAddr>,
}

impl ReadBuilder {
    pub(super) const fn new(actor: Actor, record_id: Hash) -> Self {
        Self {
            actor,
            record_id,
            sync_sources: Vec::new(),
        }
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
                let doc = LoroDoc::new();
                doc.import(&bytes)?;
                return Ok(doc);
            }
            Err(e) if e.as_str() == "record not found" => {
                // Try sync below.
            }
            Err(e) => return Err(anyhow::anyhow!("read failed: {e}")),
        }

        // Try each sync source.
        for remote in self.sync_sources {
            if self.actor.sync(self.record_id, remote).await.is_err() {
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
