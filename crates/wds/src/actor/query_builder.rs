use anyhow::Context;
use blake3::Hash;
use xdid::core::did::Did;

use crate::{actor::Actor, api::QueryRecords};

pub struct QueryBuilder {
    actor: Actor,
    creator: Option<Did>,
    schemas: Vec<Hash>,
}

impl QueryBuilder {
    pub const fn new(actor: Actor) -> Self {
        Self {
            actor,
            creator: None,
            schemas: Vec::new(),
        }
    }

    /// Filter by record creator.
    pub fn creator(mut self, did: &Did) -> Self {
        self.creator = Some(did.clone());
        self
    }

    /// Add a schema filter (AND logic with other schemas).
    pub fn schema(mut self, hash: Hash) -> Self {
        self.schemas.push(hash);
        self
    }

    /// Add multiple schema filters at once.
    pub fn schemas(mut self, hashes: &[Hash]) -> Self {
        self.schemas.extend_from_slice(hashes);
        self
    }

    /// Execute the query.
    pub async fn send(self) -> anyhow::Result<Vec<Hash>> {
        let s = self.actor.authenticate().await.context("auth")?;

        self.actor
            .api_client
            .rpc(QueryRecords {
                s,
                creator: self.creator.map(|d| d.to_string()),
                schemas: self.schemas,
            })
            .await?
            .map_err(|e| anyhow::anyhow!("query failed: {e}"))
    }
}
