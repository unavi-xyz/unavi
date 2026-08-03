use async_channel::Sender;
use bevy::prelude::*;
use bytes::Bytes;
use iroh_docs::NamespaceId;
use unavi_util::async_task::spawn_async_task;
use wds::kv;

use crate::{
    LocalBlobs,
    LocalDocs,
};

/// Creates a new writable doc, delivering its namespace on `tx`.
#[derive(Event)]
pub struct DocCreate {
    pub tx: Sender<Option<NamespaceId>>,
}

pub(crate) fn on_doc_create(trigger: On<DocCreate>, stores: Query<&LocalDocs>) {
    let Ok(docs) = stores.single() else {
        return;
    };
    let tx = trigger.event().tx.clone();
    let docs = docs.0.clone();
    spawn_async_task(async move {
        tx.send(kv::create(&docs).await.ok()).await.ok();
    });
}

/// Writes a `key -> value` entry into a doc.
#[derive(Event)]
pub struct DocSet {
    pub ns:    NamespaceId,
    pub key:   String,
    pub value: Bytes,
    pub tx:    Sender<bool>,
}

pub(crate) fn on_doc_set(trigger: On<DocSet>, stores: Query<&LocalDocs>) {
    let Ok(docs) = stores.single() else {
        return;
    };
    let event = trigger.event();
    let (ns, key, value, tx) = (
        event.ns,
        event.key.clone(),
        event.value.clone(),
        event.tx.clone(),
    );
    let docs = docs.0.clone();
    spawn_async_task(async move {
        tx.send(kv::set(&docs, ns, &key, value).await.is_ok())
            .await
            .ok();
    });
}

/// Removes a key from a doc.
#[derive(Event)]
pub struct DocDelete {
    pub ns:  NamespaceId,
    pub key: String,
    pub tx:  Sender<bool>,
}

pub(crate) fn on_doc_delete(trigger: On<DocDelete>, stores: Query<&LocalDocs>) {
    let Ok(docs) = stores.single() else {
        return;
    };
    let event = trigger.event();
    let (ns, key, tx) = (event.ns, event.key.clone(), event.tx.clone());
    let docs = docs.0.clone();
    spawn_async_task(async move {
        tx.send(kv::delete(&docs, ns, &key).await.is_ok())
            .await
            .ok();
    });
}

/// Reads the value at a key.
#[derive(Event)]
pub struct DocGet {
    pub ns:  NamespaceId,
    pub key: String,
    pub tx:  Sender<Option<Bytes>>,
}

pub(crate) fn on_doc_get(trigger: On<DocGet>, stores: Query<(&LocalDocs, &LocalBlobs)>) {
    let Ok((docs, blobs)) = stores.single() else {
        return;
    };
    let event = trigger.event();
    let (ns, key, tx) = (event.ns, event.key.clone(), event.tx.clone());
    let docs = docs.0.clone();
    let blobs = blobs.0.clone();
    spawn_async_task(async move {
        let value = kv::get(&docs, &blobs, ns, &key).await.ok().flatten();
        tx.send(value).await.ok();
    });
}

/// Lists the latest entries under a key prefix.
#[derive(Event)]
pub struct DocList {
    pub ns:     NamespaceId,
    pub prefix: String,
    pub tx:     Sender<Vec<(String, Bytes)>>,
}

pub(crate) fn on_doc_list(trigger: On<DocList>, stores: Query<(&LocalDocs, &LocalBlobs)>) {
    let Ok((docs, blobs)) = stores.single() else {
        return;
    };
    let event = trigger.event();
    let (ns, prefix, tx) = (event.ns, event.prefix.clone(), event.tx.clone());
    let docs = docs.0.clone();
    let blobs = blobs.0.clone();
    spawn_async_task(async move {
        let entries = kv::list(&docs, &blobs, ns, &prefix)
            .await
            .unwrap_or_default();
        tx.send(entries).await.ok();
    });
}
