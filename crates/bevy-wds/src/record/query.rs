use std::collections::HashSet;

use async_channel::{
    Receiver,
    Sender,
};
use bevy::prelude::*;
use blake3::Hash;
use tokio::sync::oneshot;
use unavi_util::async_task::spawn_async_task;
use wds::actor::Actor;
use xdid::core::did::Did;

use crate::{
    LocalActor,
    SyncTargets,
};

#[derive(Event)]
pub struct QueryRecord {
    pub creator: Option<String>,
    pub schemas: Vec<Hash>,
    pub cancel:  Option<oneshot::Receiver<()>>,
    pub tx:      Sender<Vec<Hash>>,
}

impl QueryRecord {
    #[must_use]
    pub fn new() -> (Self, Receiver<Vec<Hash>>, oneshot::Sender<()>) {
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let (tx, rx) = async_channel::bounded(1);
        (
            Self {
                creator: None,
                schemas: Vec::new(),
                cancel: Some(cancel_rx),
                tx,
            },
            rx,
            cancel_tx,
        )
    }
}

pub(crate) fn on_query_record(mut req: On<QueryRecord>, actor: Query<(&LocalActor, &SyncTargets)>) {
    let Ok((actor, sync_targets)) = actor.single() else {
        warn!("Unable to query records: no local actor");
        return;
    };

    let event = req.event_mut();
    let creator = event.creator.take();
    let schemas = std::mem::take(&mut event.schemas);
    let cancel = event.cancel.take();
    let tx = event.tx.clone();

    let mut actors = Vec::with_capacity(1 + sync_targets.0.len());
    actors.push(actor.0.clone());
    actors.extend(sync_targets.0.iter().cloned());

    spawn_async_task(async move {
        let cancel_fut = async move {
            match cancel {
                Some(rx) => {
                    rx.await.ok();
                }
                None => std::future::pending::<()>().await,
            }
        };

        info!(?creator, ?schemas, targets = actors.len(), "Querying");

        tokio::select! {
            () = cancel_fut => {},
            ids = query_all(creator, schemas, &actors) => {
                info!("Query result: {ids:?}");
                let _ = tx.send(ids).await;
            }
        }
    });
}

async fn query_all(creator: Option<String>, schemas: Vec<Hash>, actors: &[Actor]) -> Vec<Hash> {
    let creator_did = creator.and_then(|s| match s.parse::<Did>() {
        Ok(did) => Some(did),
        Err(err) => {
            warn!(?err, "Ignoring invalid creator DID in query filter");
            None
        }
    });

    let futures = actors
        .iter()
        .map(|a| query(creator_did.clone(), schemas.clone(), a));
    let results = n0_future::join_all(futures).await;

    let mut seen = HashSet::new();
    let mut merged = Vec::new();
    for res in results {
        match res {
            Ok(ids) => {
                for id in ids {
                    if seen.insert(id) {
                        merged.push(id);
                    }
                }
            }
            Err(err) => warn!(?err, "Query against target failed"),
        }
    }
    merged
}

async fn query(
    creator: Option<Did>,
    schemas: Vec<Hash>,
    actor: &Actor,
) -> anyhow::Result<Vec<Hash>> {
    let mut builder = actor.query();

    if let Some(did) = creator {
        builder = builder.creator(&did);
    }

    for schema in schemas {
        builder = builder.schema(schema);
    }

    builder.send().await
}
