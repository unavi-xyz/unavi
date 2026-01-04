use std::time::Duration;

use anyhow::Context;
use blake3::Hash;
use bytes::Bytes;
use iroh::EndpointId;
use loro::LoroDoc;
use tracing::warn;

use crate::{
    actor::{Actor, into_actor::IntoActor},
    record::{
        Record,
        acl::Acl,
        envelope::Envelope,
        schema::{SCHEMA_ACL, SCHEMA_RECORD, StaticSchema},
    },
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

/// Schema data for dependency tracking.
#[derive(Clone)]
pub struct SchemaData {
    pub hash: Hash,
    pub bytes: Bytes,
}

impl From<&StaticSchema> for SchemaData {
    fn from(s: &StaticSchema) -> Self {
        Self {
            hash: s.hash,
            bytes: s.bytes.clone(),
        }
    }
}

pub struct RecordBuilder {
    actor: Actor,
    doc: LoroDoc,
    schemas: Vec<SchemaData>,
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

    /// Add a schema to the record.
    /// Accepts `&StaticSchema` for builtins or [`SchemaData`] for custom schemas.
    pub fn add_schema(
        mut self,
        schema: impl Into<SchemaData>,
        f: impl Fn(&mut LoroDoc) -> anyhow::Result<()>,
    ) -> anyhow::Result<Self> {
        self.schemas.push(schema.into());
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

        // Collect all schemas (user + mandatory ACL/RECORD).
        let mut all_schemas = self.schemas.clone();
        all_schemas.push(SchemaData::from(&*SCHEMA_ACL));
        all_schemas.push(SchemaData::from(&*SCHEMA_RECORD));

        // Build record with schema hashes.
        let mut record = Record::new(did.clone());
        for schema in &all_schemas {
            record.add_schema(schema.hash);
        }
        record.save(&self.doc)?;

        let mut acl = Acl::default();
        acl.manage.push(did.clone());
        acl.write.push(did.clone());
        acl.save(&self.doc)?;

        let envelope = Envelope::all_updates(did.clone(), &self.doc)?;
        let signed = envelope.sign(self.actor.identity.signing_key())?;

        let id = record.id()?;

        // Ensure schemas exist at local actor.
        for schema in &all_schemas {
            ensure_schema_uploaded(&self.actor, schema)
                .await
                .context("ensure schema at local")?;
        }

        // Pin record locally.
        self.actor.pin_record(id, self.ttl).await?;

        // Register blob dependencies.
        let deps: Vec<_> = all_schemas.iter().map(|s| (s.hash, "schema")).collect();
        self.actor
            .register_blob_deps(id, deps)
            .await
            .context("register blob deps")?;

        // Upload the initial envelope.
        self.actor.upload_envelope(id, &signed).await?;

        // Sync to additional actors (best-effort).
        let mut sync_results = Vec::with_capacity(self.sync_targets.len());
        for target in &self.sync_targets {
            let host = *target.host();

            // Ensure schema blobs exist at remote.
            for schema in &all_schemas {
                if let Err(e) = ensure_schema_uploaded(target, schema).await {
                    warn!(host = %host, schema = %schema.hash, error = %e,
                        "failed to ensure schema at remote");
                }
            }

            let mut result = target.pin_record(id, self.ttl).await;

            if let Err(e) = &result {
                warn!(host = %host, error = %e, "failed to pin record at remote");
            } else {
                result = target.upload_envelope(id, &signed).await;
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

/// Ensures a schema exists at the given actor, uploading if missing.
async fn ensure_schema_uploaded(actor: &Actor, schema: &SchemaData) -> anyhow::Result<()> {
    let exists = actor.blob_exists(schema.hash).await?;
    if !exists {
        actor.upload_blob(schema.bytes.clone()).await?;
    }
    Ok(())
}
