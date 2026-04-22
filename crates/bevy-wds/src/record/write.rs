use std::{sync::Arc, time::Duration};

use bevy::{log::tracing::Instrument, prelude::*};
use blake3::Hash;
use loro::LoroDoc;
use smol_str::SmolStr;
use tokio::sync::{
    Notify,
    mpsc::{Receiver, Sender},
};
use unavi_util::async_task::spawn_async_task;
use wds::actor::{Actor, SchemaData};

use crate::{LocalActor, SyncTargets};

#[derive(Event, Clone)]
pub struct WriteRecord {
    /// ID of the record te write.
    /// Leave empty to create a new record.
    pub id: Option<Hash>,
    pub ttl: Option<Duration>,
    pub public: bool,
    pub schemas: Vec<SchemaDef>,
    pub cancel: Arc<Notify>,
    pub tx: Sender<Hash>,
}

#[derive(Event, Clone)]
pub struct SchemaDef {
    pub container: SmolStr,
    pub schema: SchemaData,
    pub f: Arc<dyn Fn(&mut LoroDoc) -> anyhow::Result<()> + Send + Sync>,
}

impl WriteRecord {
    #[must_use]
    pub fn new(id: Option<Hash>) -> (Self, Receiver<Hash>, Arc<Notify>) {
        let cancel = Arc::new(Notify::default());
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        (
            Self {
                id,
                ttl: None,
                public: false,
                schemas: Vec::new(),
                cancel: Arc::clone(&cancel),
                tx,
            },
            rx,
            cancel,
        )
    }
}

pub(crate) fn on_write_record(req: On<WriteRecord>, actor: Query<(&LocalActor, &SyncTargets)>) {
    let Ok((actor, sync_targets)) = actor.single() else {
        warn!("Unable to write record: no local actor");
        return;
    };

    let event = req.event().clone();
    let actor = actor.0.clone();
    let sync_targets = sync_targets.0.clone();

    spawn_async_task(async move {
        let span = info_span!("write");

        if let Err(err) = inner(event, actor, sync_targets).instrument(span).await {
            error!(?err, "failed to write record");
        }
    });
}

async fn inner(event: WriteRecord, actor: Actor, sync_targets: Vec<Actor>) -> anyhow::Result<()> {
    info!("Writing record");

    tokio::select! {
        () = event.cancel.notified() => return Ok(()),
        res = write_record(&event, &actor, &sync_targets) => {
            match res {
                Ok(res) => {
                    info!("Wrote record");
                    let _ = event.tx.send(res).await;
                    return Ok(());
                },
                Err(err)=> {
                    warn!(?err, "Could not write record");
                },
            }
        }
    }

    Ok(())
}

async fn write_record(
    event: &WriteRecord,
    actor: &Actor,
    sync_targets: &[Actor],
) -> anyhow::Result<Hash> {
    if event.id.is_none() {
        let mut builder = actor.create_record();

        if let Some(ttl) = event.ttl {
            builder = builder.ttl(ttl);
        }

        if event.public {
            builder = builder.public();
        }

        for a in sync_targets {
            builder = builder.sync_to(a.clone());
        }

        for s in &event.schemas {
            builder =
                builder.add_schema(s.container.clone(), s.schema.clone(), |doc| (s.f)(doc))?;
        }

        let res = builder.send().await?;
        Ok(res.id)
    } else {
        todo!()
    }
}
