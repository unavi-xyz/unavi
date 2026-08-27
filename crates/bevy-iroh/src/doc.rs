use async_channel::Sender;
use bevy::prelude::*;
use bytes::Bytes;
use iroh_docs::NamespaceId;
use unavi_store::store::Store;
use unavi_util::async_task::spawn_async_task;

use crate::store::LocalStore;

/// Writes a `key -> value` entry into a doc.
#[derive(Event)]
pub struct DocSet {
    pub ns:    NamespaceId,
    pub key:   String,
    pub value: Bytes,
    pub tx:    Sender<bool>,
}

pub(crate) fn on_doc_set(trigger: On<DocSet>, stores: Query<&LocalStore>) {
    let Ok(store) = stores.single().map(|s| s.0.clone()) else {
        return;
    };
    let event = trigger.event();
    let (ns, key, value, tx) = (
        event.ns,
        event.key.clone(),
        event.value.clone(),
        event.tx.clone(),
    );
    spawn_async_task(async move {
        let wrote = async { store.open(ns).await?.set(key, value).await };
        tx.send(wrote.await.is_ok()).await.ok();
    });
}

/// Reads the value at a key.
#[derive(Event)]
pub struct DocGet {
    pub ns:  NamespaceId,
    pub key: String,
    pub tx:  Sender<Option<Bytes>>,
}

pub(crate) fn on_doc_get(trigger: On<DocGet>, stores: Query<&LocalStore>) {
    let Ok(store) = stores.single().map(|s| s.0.clone()) else {
        return;
    };
    let event = trigger.event();
    let (ns, key, tx) = (event.ns, event.key.clone(), event.tx.clone());
    spawn_async_task(async move {
        tx.send(get(&store, ns, &key).await).await.ok();
    });
}

async fn get(store: &Store, ns: NamespaceId, key: &str) -> Option<Bytes> {
    let doc = store.open(ns).await.ok()?;
    let entry = doc.get(key).await.ok()??;
    doc.value(&entry).await
}

/// Lists the latest entries under a key prefix.
#[derive(Event)]
pub struct DocList {
    pub ns:     NamespaceId,
    pub prefix: String,
    pub tx:     Sender<Vec<(String, Bytes)>>,
}

pub(crate) fn on_doc_list(trigger: On<DocList>, stores: Query<&LocalStore>) {
    let Ok(store) = stores.single().map(|s| s.0.clone()) else {
        return;
    };
    let event = trigger.event();
    let (ns, prefix, tx) = (event.ns, event.prefix.clone(), event.tx.clone());
    spawn_async_task(async move {
        tx.send(list(&store, ns, &prefix).await).await.ok();
    });
}

async fn list(store: &Store, ns: NamespaceId, prefix: &str) -> Vec<(String, Bytes)> {
    let Ok(doc) = store.open(ns).await else {
        return Vec::new();
    };
    let Ok(found) = doc.list(&[prefix]).await else {
        return Vec::new();
    };

    let mut out = Vec::with_capacity(found.len());
    for entry in found {
        // No key this workspace writes is anything but UTF-8, so one that does
        // not decode names nothing a caller could have asked for.
        let Ok(key) = String::from_utf8(entry.key().to_vec()) else {
            continue;
        };
        if let Some(value) = doc.value(&entry).await {
            out.push((key, value));
        }
    }
    out
}
