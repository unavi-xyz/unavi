use anyhow::Context;
use async_channel::Sender;
use bevy::prelude::*;
use bytes::Bytes;
use iroh_blobs::api::blobs::Blobs;
use iroh_docs::{
    NamespaceId,
    protocol::Docs,
};
use unavi_store::entries::{
    self,
    Write,
};
use unavi_util::async_task::spawn_async_task;

use crate::store::{
    LocalBlobs,
    LocalDocs,
};

/// Writes a `key -> value` entry into a doc.
#[derive(Event)]
pub struct DocSet {
    pub ns:    NamespaceId,
    pub key:   String,
    pub value: Bytes,
    pub tx:    Sender<bool>,
}

pub(crate) fn on_doc_set(trigger: On<DocSet>, stores: Query<(&LocalDocs, &LocalBlobs)>) {
    let Ok((docs, blobs)) = stores.single() else {
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
    let blobs = blobs.0.clone();
    spawn_async_task(async move {
        tx.send(set(&docs, &blobs, ns, key, value).await.is_ok())
            .await
            .ok();
    });
}

async fn set(
    docs: &Docs,
    blobs: &Blobs,
    ns: NamespaceId,
    key: String,
    value: Bytes,
) -> anyhow::Result<()> {
    let doc = docs.api().open(ns).await?.context("doc not open")?;
    let author = docs.api().author_default().await?;
    entries::apply(&doc, blobs, author, [Write::Bytes { key, value }]).await
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
        tx.send(get(&docs, &blobs, ns, &key).await).await.ok();
    });
}

async fn get(docs: &Docs, blobs: &Blobs, ns: NamespaceId, key: &str) -> Option<Bytes> {
    let doc = docs.api().open(ns).await.ok()??;
    let entry = entries::get(&doc, key).await.ok()??;
    entries::value(blobs, &entry).await
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
        tx.send(list(&docs, &blobs, ns, &prefix).await).await.ok();
    });
}

async fn list(docs: &Docs, blobs: &Blobs, ns: NamespaceId, prefix: &str) -> Vec<(String, Bytes)> {
    let Ok(Some(doc)) = docs.api().open(ns).await else {
        return Vec::new();
    };
    let Ok(found) = entries::list(&doc, &[prefix]).await else {
        return Vec::new();
    };

    let mut out = Vec::with_capacity(found.len());
    for entry in found {
        if let Some(value) = entries::value(blobs, &entry).await {
            out.push((entry.key, value));
        }
    }
    out
}
