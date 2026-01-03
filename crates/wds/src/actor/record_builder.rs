use std::time::Duration;

use blake3::Hash;
use iroh::EndpointId;
use loro::LoroDoc;
use tracing::warn;

use crate::{
    actor::{Actor, into_actor::IntoActor},
    record::{Record, acl::Acl, envelope::Envelope},
    signed_bytes::Signable,
};

const DEFAULT_PIN_TTL: Duration = Duration::from_hours(1);

/// Result of creating a record.
#[derive(Debug)]
pub struct RecordResult {
    pub id: Hash,
    pub doc: LoroDoc,
    pub sync_results: Vec<(EndpointId, anyhow::Result<()>)>,
}

pub struct RecordBuilder {
    actor: Actor,
    doc: LoroDoc,
    schemas: Vec<Hash>,
    ttl: Duration,
    sync_targets: Vec<Actor>,
}

impl RecordBuilder {
    pub fn new(actor: Actor) -> Self {
        let doc = LoroDoc::new();
        Self {
            actor,
            doc,
            schemas: Vec::new(),
            ttl: DEFAULT_PIN_TTL,
            sync_targets: Vec::new(),
        }
    }

    pub fn add_schema(
        mut self,
        id: Hash,
        f: impl Fn(&mut LoroDoc) -> anyhow::Result<()>,
    ) -> anyhow::Result<Self> {
        self.schemas.push(id);
        f(&mut self.doc)?;
        Ok(self)
    }

    pub const fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    pub fn sync_to(mut self, actor: impl IntoActor) -> Self {
        if let Some(a) = actor.into_actor() {
            self.sync_targets.push(a);
        }
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

        // Sync to additional actors (best-effort).
        let mut sync_results = Vec::with_capacity(self.sync_targets.len());
        for target in self.sync_targets {
            let host = *target.host();
            let result = target.pin_record(id, self.ttl).await;
            if let Err(e) = &result {
                warn!(host = %host, error = %e, "failed to pin record at remote");
            } else {
                // TODO initiate sync
            }
            sync_results.push((host, result));
        }

        Ok(RecordResult {
            id,
            doc: self.doc,
            sync_results,
        })
    }
}
