use std::sync::Arc;

use futures::StreamExt;
use iroh_docs::{
    NamespaceId,
    api::Doc,
    engine::LiveEvent,
    store::Query,
};
use irpc::WithChannels;
use n0_future::task::AbortOnDropHandle;
use rusqlite::{
    Connection,
    params,
};

use crate::{
    HostedDoc,
    StoreContext,
    control::{
        ControlService,
        HostDoc,
        UnhostDoc,
        authenticate,
    },
    error::ApiError,
    quota::{
        ensure_quota_exists,
        release_bytes,
        reserve_bytes,
    },
};

pub async fn host_doc(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<HostDoc, ControlService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);
    let did_str = did.to_string();
    let ns_str = inner.ns.to_string();

    let doc = crate::docs::ensure_open(&ctx.docs, inner.ns).await?;
    let size = doc_size(&doc).await?;

    let admitted = ctx
        .db
        .call_mut({
            let did_str = did_str.clone();
            let ns_str = ns_str.clone();
            move |conn| {
                let tx = conn.transaction()?;
                ensure_quota_exists(&tx, &did_str)?;
                // A repeat request inserts nothing, which is what keeps it from
                // charging twice or spawning a second meter.
                let rows = tx.execute(
                    "INSERT OR IGNORE INTO hosted_docs (ns, owner, bytes_used) VALUES (?, ?, 0)",
                    params![&ns_str, &did_str],
                )?;
                if rows == 0 {
                    return Ok(true);
                }
                if reserve_bytes(&tx, &did_str, size).is_err() {
                    return Ok(false);
                }
                tx.execute(
                    "UPDATE hosted_docs SET bytes_used = ? WHERE owner = ? AND ns = ?",
                    params![size, &did_str, &ns_str],
                )?;
                tx.commit()?;
                Ok(true)
            }
        })
        .await?;

    if !admitted {
        tx.send(Err(ApiError::QuotaExceeded)).await?;
        return Ok(());
    }

    // Agreeing to host means agreeing to replicate. Importing alone leaves the
    // namespace out of the sync set, where the host rejects every incoming
    // request with `NotFound` — availability being the entire point of hosting.
    doc.start_sync(Vec::new()).await?;

    let meter = AbortOnDropHandle::new(n0_future::task::spawn(meter_doc(
        Arc::clone(&ctx),
        inner.ns,
        doc.clone(),
    )));
    // One meter per namespace, not per owner. Losing this race means the
    // namespace is already hosted: dropping the loser aborts its meter, and
    // its doc handle is closed so the replica's open count stays at the one
    // handle hosting actually holds.
    if let Err((_, loser)) = ctx
        .hosted
        .insert_async(inner.ns, HostedDoc { doc, _meter: meter })
        .await
        && let Err(err) = loser.doc.close().await
    {
        tracing::debug!(ns = %inner.ns, "closing duplicate doc handle failed: {err}");
    }

    tx.send(Ok(())).await?;
    Ok(())
}

pub async fn unhost_doc(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<UnhostDoc, ControlService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);
    let did_str = did.to_string();
    let ns_str = inner.ns.to_string();

    // `None` when the caller was not hosting this namespace at all; `Some(n)`
    // carries how many *other* owners still host it.
    let remaining = ctx
        .db
        .call_mut(move |conn| {
            let tx = conn.transaction()?;
            let bytes_used: Option<i64> = tx
                .query_row(
                    "SELECT bytes_used FROM hosted_docs WHERE owner = ? AND ns = ?",
                    params![&did_str, &ns_str],
                    |row| row.get(0),
                )
                .ok();
            let Some(bytes_used) = bytes_used else {
                return Ok(None);
            };
            tx.execute(
                "DELETE FROM hosted_docs WHERE owner = ? AND ns = ?",
                params![&did_str, &ns_str],
            )?;
            if bytes_used > 0 {
                release_bytes(&tx, &did_str, bytes_used)?;
            }
            let others: i64 = tx.query_row(
                "SELECT COUNT(*) FROM hosted_docs WHERE ns = ?",
                params![&ns_str],
                |row| row.get(0),
            )?;
            tx.commit()?;
            Ok(Some(others))
        })
        .await?;

    // Denying rather than silently succeeding keeps this from doubling as a
    // probe for which namespaces the node holds.
    let Some(remaining) = remaining else {
        tx.send(Err(ApiError::AccessDenied)).await?;
        return Ok(());
    };

    if remaining == 0 {
        stop_hosting(&ctx, inner.ns).await;
    }

    tx.send(Ok(())).await?;
    Ok(())
}

/// Ends replication of `ns` and releases this node's own handle on it.
///
/// The replica is only deleted if nothing else still holds it open; another
/// holder is a legitimate in-process user, not something to tear out from
/// under.
async fn stop_hosting(ctx: &StoreContext, ns: NamespaceId) {
    let Some((_, hosted)) = ctx.hosted.remove_async(&ns).await else {
        return;
    };
    if let Err(err) = hosted.doc.leave().await {
        tracing::debug!(%ns, "leaving sync failed: {err}");
    }
    if let Err(err) = hosted.doc.close().await {
        tracing::debug!(%ns, "closing doc failed: {err}");
        return;
    }
    if let Err(err) = ctx.docs.api().drop_doc(ns).await {
        tracing::debug!(%ns, "replica still held open, not dropped: {err}");
    }
}

async fn doc_size(doc: &Doc) -> anyhow::Result<i64> {
    let mut total: i64 = 0;
    let entries = doc.get_many(Query::all()).await?;
    let mut entries = std::pin::pin!(entries);
    while let Some(entry) = entries.next().await {
        total = total.saturating_add(i64::try_from(entry?.content_len()).unwrap_or(i64::MAX));
    }
    Ok(total)
}

/// Meters a hosted doc against the quota of every owner hosting it, charging
/// each live insert. On overflow it stops replicating the doc rather than
/// letting it grow uncharged.
async fn meter_doc(ctx: Arc<StoreContext>, ns: NamespaceId, doc: Doc) {
    let mut events = match doc.subscribe().await {
        Ok(events) => events,
        Err(err) => {
            tracing::warn!(%ns, "failed subscribing to hosted doc: {err}");
            return;
        }
    };

    while let Some(event) = events.next().await {
        let len = match event {
            Ok(LiveEvent::InsertLocal { entry } | LiveEvent::InsertRemote { entry, .. }) => {
                entry.content_len()
            }
            Ok(_) => continue,
            Err(err) => {
                tracing::warn!(%ns, "hosted doc event error: {err}");
                continue;
            }
        };

        if let Err(err) = charge(&ctx, ns, i64::try_from(len).unwrap_or(i64::MAX)).await {
            tracing::error!(%ns, "hosted doc over quota, ending replication: {err}");
            if let Err(err) = doc.leave().await {
                tracing::error!(%ns, "failed to stop replicating an over-quota doc: {err}");
            }
            return;
        }
    }
}

/// Reserves `len` bytes against every owner hosting `ns` and adds them to each
/// one's ledger, in a single transaction.
async fn charge(ctx: &StoreContext, ns: NamespaceId, len: i64) -> anyhow::Result<()> {
    if len == 0 {
        return Ok(());
    }
    let ns = ns.to_string();
    ctx.db
        .call_mut(move |conn| {
            let tx = conn.transaction()?;
            for owner in owners_of(&tx, &ns)? {
                if reserve_bytes(&tx, &owner, len).is_err() {
                    anyhow::bail!("quota exceeded for {owner}");
                }
            }
            tx.execute(
                "UPDATE hosted_docs SET bytes_used = bytes_used + ? WHERE ns = ?",
                params![len, &ns],
            )?;
            tx.commit()?;
            Ok(())
        })
        .await
}

fn owners_of(conn: &Connection, ns: &str) -> anyhow::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT owner FROM hosted_docs WHERE ns = ?")?;
    let rows = stmt.query_map(params![ns], |row| row.get::<_, String>(0))?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}
