use std::{sync::Arc, time::Duration};

use bevy::{log::tracing::Instrument, prelude::*};
use blake3::Hash;
use iroh::EndpointAddr;
use loro::LoroDoc;
use tokio::sync::{
    Notify,
    mpsc::{Receiver, Sender},
};
use wds::actor::Actor;

use crate::{LocalActor, SyncTargets};

#[derive(Event, Clone)]
pub struct ReadRecord {
    pub id: Hash,
    pub ttl: Option<Duration>,
    pub backoff_secs: u64,
    pub retries: usize,
    pub cancel: Arc<Notify>,
    pub tx: Sender<LoroDoc>,
}

impl ReadRecord {
    #[must_use]
    pub fn new(id: Hash) -> (Self, Receiver<LoroDoc>, Arc<Notify>) {
        let cancel = Arc::new(Notify::default());
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        (
            Self {
                id,
                ttl: None,
                backoff_secs: 4,
                retries: 3,
                cancel: Arc::clone(&cancel),
                tx,
            },
            rx,
            cancel,
        )
    }
}

pub(crate) fn on_get_record(req: On<ReadRecord>, actor: Query<(&LocalActor, &SyncTargets)>) {
    let Ok((actor, sync_targets)) = actor.single() else {
        warn!("unable to get record: no local actor");
        return;
    };

    let event = req.event().clone();
    let actor = actor.0.clone();
    let sync_targets = sync_targets.0.iter().map(|a| a.host().clone()).collect();

    unavi_wasm_compat::spawn_thread(async move {
        let span = info_span!("read", id = %event.id);

        if let Err(err) = inner(event, actor, sync_targets).instrument(span).await {
            error!(?err, "failed to get record");
        }
    });
}

async fn inner(
    event: ReadRecord,
    actor: Actor,
    sync_targets: Vec<EndpointAddr>,
) -> anyhow::Result<()> {
    info!("reading record");

    let mut n = 0;
    let mut delay_secs = event.backoff_secs;

    while n <= event.retries {
        tokio::select! {
            () = event.cancel.notified() => return Ok(()),
            res = read_record(&event, &actor, &sync_targets) => {
                match res {
                    Ok(res) => {
                        info!("got record");
                        let _ = event.tx.send(res).await;
                        return Ok(());
                    },
                    Err(err)=> {
                        warn!(?err, "could not read record ({n}/{})", event.retries);
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
    event: &ReadRecord,
    actor: &Actor,
    sync_targets: &[EndpointAddr],
) -> anyhow::Result<LoroDoc> {
    let mut builder = actor.read(event.id);

    if let Some(ttl) = event.ttl {
        builder = builder.ttl(ttl);
    }

    for e in sync_targets {
        builder = builder.sync_from(e.clone());
    }

    let res = builder.send().await?;

    Ok(res)
}
