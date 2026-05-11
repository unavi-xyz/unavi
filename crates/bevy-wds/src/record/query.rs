use async_channel::{Receiver, Sender};
use bevy::prelude::*;
use blake3::Hash;
use tokio::sync::oneshot;
use unavi_util::async_task::spawn_async_task;
use wds::actor::Actor;
use xdid::core::did::Did;

use crate::LocalActor;

#[derive(Event)]
pub struct QueryRecord {
    pub creator: Option<String>,
    pub schemas: Vec<Hash>,
    pub cancel: Option<oneshot::Receiver<()>>,
    pub tx: Sender<Vec<Hash>>,
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

pub(crate) fn on_query_record(mut req: On<QueryRecord>, actor: Query<&LocalActor>) {
    let Ok(actor) = actor.single() else {
        warn!("Unable to query records: no local actor");
        return;
    };

    let event = req.event_mut();
    let creator = event.creator.take();
    let schemas = std::mem::take(&mut event.schemas);
    let cancel = event.cancel.take();
    let tx = event.tx.clone();

    let actor = actor.0.clone();

    spawn_async_task(async move {
        let cancel_fut = async move {
            match cancel {
                Some(rx) => {
                    rx.await.ok();
                }
                None => std::future::pending::<()>().await,
            }
        };

        tokio::select! {
            () = cancel_fut => {},
            res = query(creator, schemas, &actor) => {
                match res {
                    Ok(ids) => { let _ = tx.send(ids).await; }
                    Err(err) => warn!(?err, "Could not query records"),
                }
            }
        }
    });
}

async fn query(
    creator: Option<String>,
    schemas: Vec<Hash>,
    actor: &Actor,
) -> anyhow::Result<Vec<Hash>> {
    let mut builder = actor.query();

    if let Some(s) = creator {
        match s.parse::<Did>() {
            Ok(did) => builder = builder.creator(&did),
            Err(err) => warn!(?err, "Ignoring invalid creator DID in query filter"),
        }
    }

    for schema in schemas {
        builder = builder.schema(schema);
    }

    builder.send().await
}
