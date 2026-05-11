use std::time::Duration;

use async_channel::{Receiver, Sender};
use bevy::{log::tracing::Instrument, prelude::*};
use blake3::Hash;
use iroh_base::EndpointAddr;
use loro::LoroDoc;
use tokio::sync::oneshot;
use unavi_util::async_task::spawn_async_task;
use wds::actor::Actor;

use crate::{LocalActor, SyncTargets};

#[derive(Event)]
pub struct ReadRecord {
    pub id: Hash,
    pub ttl: Option<Duration>,
    pub backoff_secs: u64,
    pub retries: usize,
    pub cancel: Option<oneshot::Receiver<()>>,
    pub tx: Sender<LoroDoc>,
}

impl ReadRecord {
    #[must_use]
    pub fn new(id: Hash) -> (Self, Receiver<LoroDoc>, oneshot::Sender<()>) {
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let (tx, rx) = async_channel::bounded(1);
        (
            Self {
                id,
                ttl: None,
                backoff_secs: 4,
                retries: 3,
                cancel: Some(cancel_rx),
                tx,
            },
            rx,
            cancel_tx,
        )
    }
}

pub(crate) fn on_read_record(mut req: On<ReadRecord>, actor: Query<(&LocalActor, &SyncTargets)>) {
    let Ok((actor, sync_targets)) = actor.single() else {
        warn!("Unable to read record: no local actor");
        return;
    };

    let event = req.event_mut();
    let id = event.id;
    let ttl = event.ttl;
    let backoff_secs = event.backoff_secs;
    let retries = event.retries;
    let cancel = event.cancel.take();
    let tx = event.tx.clone();

    let actor = actor.0.clone();
    let sync_targets = sync_targets.0.iter().map(|a| a.host().clone()).collect();

    spawn_async_task(async move {
        let span = info_span!("read", id = %id);

        if let Err(err) = inner(
            id,
            ttl,
            backoff_secs,
            retries,
            cancel,
            tx,
            actor,
            sync_targets,
        )
        .instrument(span)
        .await
        {
            error!(?err, "Failed to read record");
        }
    });
}

async fn inner(
    id: Hash,
    ttl: Option<Duration>,
    backoff_secs: u64,
    retries: usize,
    cancel: Option<oneshot::Receiver<()>>,
    tx: Sender<LoroDoc>,
    actor: Actor,
    sync_targets: Vec<EndpointAddr>,
) -> anyhow::Result<()> {
    info!("Reading record");

    let cancel_fut = async move {
        match cancel {
            Some(rx) => {
                rx.await.ok();
            }
            None => std::future::pending::<()>().await,
        }
    };
    tokio::pin!(cancel_fut);

    let mut n = 0;
    let mut delay_secs = backoff_secs;

    while n <= retries {
        tokio::select! {
            () = &mut cancel_fut => return Ok(()),
            res = read_record(id, ttl, &actor, &sync_targets) => {
                match res {
                    Ok(res) => {
                        info!(%id, "Got record");
                        let _ = tx.send(res).await;
                        return Ok(());
                    },
                    Err(err) => {
                        warn!(?err, "Could not read record ({n}/{retries})");
                        n0_future::time::sleep(Duration::from_secs(delay_secs)).await;
                        delay_secs = delay_secs.wrapping_mul(2);
                    },
                }
            }
        }

        n += 1;
    }

    Ok(())
}

async fn read_record(
    id: Hash,
    ttl: Option<Duration>,
    actor: &Actor,
    sync_targets: &[EndpointAddr],
) -> anyhow::Result<LoroDoc> {
    let mut builder = actor.read(id);

    if let Some(ttl) = ttl {
        builder = builder.ttl(ttl);
    }

    for e in sync_targets {
        builder = builder.sync_from(e.clone());
    }

    let res = builder.send().await?;

    Ok(res)
}
