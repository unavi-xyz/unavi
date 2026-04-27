use std::time::Duration;

use async_channel::{Receiver, Sender};
use bevy::{log::tracing::Instrument, prelude::*};
use blake3::Hash;
use loro::LoroDoc;
use smol_str::SmolStr;
use tokio::sync::oneshot;
use unavi_util::async_task::spawn_async_task;
use wds::actor::{Actor, SchemaData};

use crate::{LocalActor, SyncTargets};

#[derive(Event)]
pub struct WriteRecord {
    /// ID of the record to write.
    /// Leave empty to create a new record.
    pub id: Option<Hash>,
    pub ttl: Option<Duration>,
    pub public: bool,
    pub schemas: Vec<SchemaDef>,
    pub cancel: Option<oneshot::Receiver<()>>,
    pub tx: Sender<Hash>,
}

#[derive(Event, Clone)]
pub struct SchemaDef {
    pub container: SmolStr,
    pub schema: SchemaData,
    pub f: std::sync::Arc<dyn Fn(&mut LoroDoc) -> anyhow::Result<()> + Send + Sync>,
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

    spawn_async_task(async move {
        let span = info_span!("write");

        if let Err(err) = inner(id, ttl, public, schemas, cancel, tx, actor, sync_targets)
            .instrument(span)
            .await
        {
            error!(?err, "failed to write record");
        }
    });
}

async fn inner(
    id: Option<Hash>,
    ttl: Option<Duration>,
    public: bool,
    schemas: Vec<SchemaDef>,
    cancel: Option<oneshot::Receiver<()>>,
    tx: Sender<Hash>,
    actor: Actor,
    sync_targets: Vec<Actor>,
) -> anyhow::Result<()> {
    info!("Writing record");

    let cancel_fut = async move {
        match cancel {
            Some(rx) => {
                rx.await.ok();
            }
            None => std::future::pending::<()>().await,
        }
    };
    tokio::select! {
        () = cancel_fut => return Ok(()),
        res = write_record(id, ttl, public, &schemas, &actor, &sync_targets) => {
            match res {
                Ok(res) => {
                    info!("Wrote record");
                    let _ = tx.send(res).await;
                    return Ok(());
                },
                Err(err) => {
                    warn!(?err, "Could not write record");
                },
            }
        }
    }

    Ok(())
}

async fn write_record(
    id: Option<Hash>,
    ttl: Option<Duration>,
    public: bool,
    schemas: &[SchemaDef],
    actor: &Actor,
    sync_targets: &[Actor],
) -> anyhow::Result<Hash> {
    if id.is_none() {
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
            builder =
                builder.add_schema(s.container.clone(), s.schema.clone(), |doc| (s.f)(doc))?;
        }

        let res = builder.send().await?;
        Ok(res.id)
    } else {
        todo!()
    }
}
