use std::time::Duration;

use anyhow::Context;
use async_channel::{
    Receiver,
    Sender,
};
use bevy::{
    log::tracing::Instrument,
    prelude::*,
};
use blake3::Hash;
use loro::LoroDoc;
use smol_str::SmolStr;
use tokio::sync::oneshot;
use unavi_util::async_task::spawn_async_task;
use wds::{
    actor::{
        Actor,
        SchemaData,
    },
    surg::acl::Acl,
};

use crate::{
    LocalActor,
    SyncTargets,
};

#[derive(Event)]
pub struct WriteRecord {
    /// ID of the record to write.
    /// Leave empty to create a new record.
    pub id:      Option<Hash>,
    pub ttl:     Option<Duration>,
    pub public:  bool,
    pub schemas: Vec<SchemaDef>,
    pub cancel:  Option<oneshot::Receiver<()>>,
    pub tx:      Sender<Hash>,
}

#[derive(Event, Clone)]
pub struct SchemaDef {
    pub container: SmolStr,
    pub schema:    SchemaData,
    pub f:         std::sync::Arc<dyn Fn(&mut LoroDoc) -> anyhow::Result<()> + Send + Sync>,
}

impl WriteRecord {
    #[must_use]
    pub fn new(id: Option<Hash>) -> (Self, Receiver<Hash>, oneshot::Sender<()>) {
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let (tx, rx) = async_channel::bounded(1);
        (
            Self {
                id,
                ttl: None,
                public: false,
                schemas: Vec::new(),
                cancel: Some(cancel_rx),
                tx,
            },
            rx,
            cancel_tx,
        )
    }
}

pub(crate) fn on_write_record(mut req: On<WriteRecord>, actor: Query<(&LocalActor, &SyncTargets)>) {
    let Ok((actor, sync_targets)) = actor.single() else {
        warn!("Unable to write record: no local actor");
        return;
    };

    let event = req.event_mut();
    let id = event.id;
    let ttl = event.ttl;
    let public = event.public;
    let schemas = event.schemas.clone();
    let cancel = event.cancel.take();
    let tx = event.tx.clone();

    let actor = actor.0.clone();
    let sync_targets = sync_targets.0.clone();

    let span = info_span!("write");
    spawn_async_task(
        async move {
            info!(?id, "Writing record");

            let cancel_fut = async move {
                match cancel {
                    Some(rx) => {
                        rx.await.ok();
                    }
                    None => std::future::pending::<()>().await,
                }
            };

            tokio::select! {
                () = cancel_fut => {
                    info!(?id, "Cancelled");
                },
                res = write_record(id, ttl, public, &schemas, &actor, &sync_targets) => {
                    match res {
                        Ok(id) => {
                            info!(%id, "Wrote record");
                            let _ = tx.send(id).await;
                        },
                        Err(err) => {
                            warn!(?err, "Could not write record");
                        },
                    }
                }
            };
        }
        .instrument(span),
    );
}

async fn write_record(
    id: Option<Hash>,
    ttl: Option<Duration>,
    public: bool,
    schemas: &[SchemaDef],
    actor: &Actor,
    sync_targets: &[Actor],
) -> anyhow::Result<Hash> {
    let Some(id) = id else {
        return create_record(ttl, public, schemas, actor, sync_targets).await;
    };

    update_record(id, ttl, public, schemas, actor)
        .await
        .context("update at local actor")?;
    for target in sync_targets {
        if let Err(err) = update_record(id, ttl, public, schemas, target).await {
            warn!(host = %target.host().id, ?err, "failed to update record at sync target");
        }
    }
    Ok(id)
}

async fn create_record(
    ttl: Option<Duration>,
    public: bool,
    schemas: &[SchemaDef],
    actor: &Actor,
    sync_targets: &[Actor],
) -> anyhow::Result<Hash> {
    let mut builder = actor.create_record();

    if let Some(ttl) = ttl {
        builder = builder.ttl(ttl);
    }

    if public {
        builder = builder.public();
    }

    for a in sync_targets {
        builder = builder.sync_to(a.clone());
    }

    for s in schemas {
        builder = builder.add_schema(s.container.clone(), s.schema.clone(), |doc| (s.f)(doc))?;
    }

    let res = builder.send().await?;
    Ok(res.id)
}

/// Applies the schema mutations to an existing record at `actor` and uploads
/// the diff. Reads the host's current version first so only the new ops are
/// sent.
async fn update_record(
    id: Hash,
    ttl: Option<Duration>,
    public: bool,
    schemas: &[SchemaDef],
    actor: &Actor,
) -> anyhow::Result<()> {
    let mut doc = actor.read(id).send().await.context("read record")?;
    let from = doc.oplog_vv();

    for s in schemas {
        (s.f)(&mut doc).context("apply schema update")?;
    }

    if public {
        let mut acl = Acl::load(&doc).context("load acl")?;
        if !acl.public {
            acl.public = true;
            acl.save(&doc).context("save acl")?;
        }
    }

    doc.commit();
    actor
        .update_record(id, &doc, from)
        .await
        .context("upload update")?;

    if let Some(ttl) = ttl {
        actor.pin_record(id, ttl).await.context("repin")?;
    }

    Ok(())
}
