use std::time::Duration;

use anyhow::Context;
use blake3::Hash;
use bytes::Bytes;
use iroh::EndpointId;
use loro::LoroDoc;
use tracing::warn;

use wired_schemas::{Acl, Record, SCHEMA_ACL, SCHEMA_RECORD, StaticSchema};

use crate::{
    actor::{Actor, into_actor::IntoActor},
    record::envelope::Envelope,
    signed_bytes::Signable,
};

pub(super) const DEFAULT_PIN_TTL: Duration = Duration::from_hours(1);

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
    pub container: smol_str::SmolStr,
    pub hash: Hash,
    pub bytes: Bytes,
}

impl From<&StaticSchema> for SchemaData {
    fn from(s: &StaticSchema) -> Self {
        Self {
            container: "unknown".into(), // Will be set by add_schema
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
    is_public: bool,
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
            is_public: false,
        }
    }

    /// Mark the record as public (anyone can read).
    ///
    /// By default, records are private (only the creator can read).
    pub const fn public(mut self) -> Self {
        self.is_public = true;
        self
    }

    /// Add a schema to the record.
    /// Accepts `&StaticSchema` for builtins or [`SchemaData`] for custom schemas.
    pub fn add_schema(
        mut self,
        container: impl Into<smol_str::SmolStr>,
        schema: impl Into<SchemaData>,
        f: impl Fn(&mut LoroDoc) -> anyhow::Result<()>,
    ) -> anyhow::Result<Self> {
        let mut schema_data = schema.into();
        schema_data.container = container.into();
        self.schemas.push(schema_data);
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
        let mut acl_schema = SchemaData::from(&*SCHEMA_ACL);
        acl_schema.container = "acl".into();
        all_schemas.push(acl_schema);
        let mut record_schema = SchemaData::from(&*SCHEMA_RECORD);
        record_schema.container = "record".into();
        all_schemas.push(record_schema);

        // Build record with schema hashes.
        let mut record = Record::new(did.clone());
        for schema in &all_schemas {
            record.add_schema(schema.container.clone(), schema.hash);
        }
        record.save(&self.doc)?;

        let mut acl = Acl {
            public: self.is_public,
            ..Default::default()
        };
        acl.manage.push(did.clone());
        acl.write.push(did.clone());
        if !self.is_public {
            acl.read.push(did.clone());
        }
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
