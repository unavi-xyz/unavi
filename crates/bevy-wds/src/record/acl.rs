use std::sync::Arc;

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
use unavi_util::async_task::spawn_async_task;

use crate::LocalActor;

#[derive(Event)]
pub struct SetRecordPublic {
    pub id:     Hash,
    pub doc:    Arc<LoroDoc>,
    pub public: bool,
    pub tx:     Sender<Result<(), String>>,
}

impl SetRecordPublic {
    #[must_use]
    pub fn new(id: Hash, doc: Arc<LoroDoc>, public: bool) -> (Self, Receiver<Result<(), String>>) {
        let (tx, rx) = async_channel::bounded(1);
        (
            Self {
                id,
                doc,
                public,
                tx,
            },
            rx,
        )
    }
}

pub(crate) fn on_set_record_public(req: On<SetRecordPublic>, actors: Query<&LocalActor>) {
    let Ok(actor) = actors.single().map(|a| a.0.clone()) else {
        warn!("Unable to set record ACL: no local actor");
        return;
    };

    let event = req.event();
    let id = event.id;
    let doc = Arc::clone(&event.doc);
    let public = event.public;
    let tx = event.tx.clone();

    let span = info_span!("set_record_public", %id);
    spawn_async_task(
        async move {
            let res = actor
                .set_record_public(id, &doc, public)
                .await
                .map_err(|err| err.to_string());
            let _ = tx.send(res).await;
        }
        .instrument(span),
    );
}
