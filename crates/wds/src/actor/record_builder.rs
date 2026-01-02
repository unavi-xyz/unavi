use std::time::Duration;

use anyhow::Context;
use blake3::Hash;
use iroh::EndpointId;
use loro::LoroDoc;
use time::OffsetDateTime;
use tracing::warn;

use crate::{
    actor::Actor,
    api::PinRecord,
    record::{Record, acl::Acl, envelope::Envelope},
    signed_bytes::Signable,
};

const DEFAULT_PIN_TTL: Duration = Duration::from_hours(1);

/// Result of creating a record.
#[derive(Debug)]
pub struct RecordResult {
    pub id: Hash,
    pub doc: LoroDoc,
    /// Results of additional pin operations at other hosts.
    pub additional_pins: Vec<(EndpointId, anyhow::Result<()>)>,
}

pub struct RecordBuilder {
    actor: Actor,
    doc: LoroDoc,
    schemas: Vec<Hash>,
    ttl: Duration,
    additional_pins: Vec<Actor>,
}

impl RecordBuilder {
    pub fn new(actor: Actor) -> Self {
        let doc = LoroDoc::new();
        Self {
            actor,
            doc,
            schemas: Vec::new(),
            ttl: DEFAULT_PIN_TTL,
            additional_pins: Vec::new(),
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

    /// Also pin the record at another actor's host after creation.
    pub fn with_pin_at(mut self, actor: &Actor) -> Self {
        self.additional_pins.push(actor.clone());
        self
    }

    pub async fn send(self) -> anyhow::Result<RecordResult> {
        let did = self.actor.identity.did();

        let mut record = Record::new(did.clone());
        for schema in self.schemas {
            record.add_schema(schema);
        }
        record.save(&self.doc)?;

        let mut acl = Acl::default();
        acl.manage.push(did.clone());
        acl.write.push(did.clone());
        acl.save(&self.doc)?;

        let envelope = Envelope::all_updates(did.clone(), &self.doc)?;
        let signed = envelope.sign(self.actor.identity.signing_key())?;

        let id = record.id()?;

        // Pin record locally.
        self.actor.pin_record(id, self.ttl).await?;

        // Upload the initial envelope.
        self.actor.upload_envelope(id, signed).await?;

        // Pin at additional hosts (best-effort).
        let mut additional_results = Vec::with_capacity(self.additional_pins.len());
        for remote_actor in self.additional_pins {
            let host = *remote_actor.host();
            let result = remote_actor.pin_record(id, self.ttl).await;
            if let Err(e) = &result {
                warn!(host = %host, error = %e, "failed to pin record at remote");
            }
            additional_results.push((host, result));
        }

        Ok(RecordResult {
            id,
            doc: self.doc,
            additional_pins: additional_results,
        })
    }
}
