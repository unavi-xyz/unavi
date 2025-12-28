use std::time::Duration;

use anyhow::Context;
use blake3::Hash;
use loro::LoroDoc;
use time::OffsetDateTime;

use crate::{
    actor::Actor,
    api::PinRecord,
    record::{Record, acl::Acl, envelope::Envelope},
    signed_bytes::Signable,
};

const DEFAULT_PIN_TTL: Duration = Duration::from_hours(1);

pub struct RecordBuilder {
    actor: Actor,
    doc: LoroDoc,
    schemas: Vec<Hash>,
    ttl: Duration,
}

impl RecordBuilder {
    pub fn new(actor: Actor) -> Self {
        let doc = LoroDoc::new();
        Self {
            actor,
            doc,
            schemas: Vec::new(),
            ttl: DEFAULT_PIN_TTL,
        }
    }

    pub fn with_schema(
        mut self,
        id: Hash,
        f: impl Fn(&mut LoroDoc) -> anyhow::Result<()>,
    ) -> anyhow::Result<Self> {
        self.schemas.push(id);
        f(&mut self.doc)?;
        Ok(self)
    }

    pub const fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    pub async fn send(self) -> anyhow::Result<(Hash, LoroDoc)> {
        let mut record = Record::new(self.actor.did.clone());
        for schema in self.schemas {
            record.add_schema(schema);
        }
        record.save(&self.doc)?;

        let mut acl = Acl::default();
        acl.manage.push(self.actor.did.clone());
        acl.write.push(self.actor.did.clone());
        acl.save(&self.doc)?;

        let envelope = Envelope::all_updates(self.actor.did.clone(), &self.doc)?;
        let signed = envelope.sign(&self.actor.signing_key)?;

        let id = record.id()?;

        // Pin record.
        let s = self.actor.authenticate().await.context("auth")?;

        self.actor
            .api_client
            .rpc(PinRecord {
                s,
                id,
                expires: (OffsetDateTime::now_utc() + self.ttl).unix_timestamp(),
            })
            .await?
            .map_err(|e| anyhow::anyhow!("record pin failed: {e}"))?;

        // Upload the initial envelope.
        self.actor.upload_envelope(id, signed).await?;

        Ok((id, self.doc))
    }
}
