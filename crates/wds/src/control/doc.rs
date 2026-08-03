use std::sync::Arc;

use futures::StreamExt;
use iroh_docs::{
    NamespaceId,
    api::Doc,
    engine::LiveEvent,
    store::Query,
};
use irpc::WithChannels;
use rusqlite::params;
use xdid::core::did::Did;

use crate::{
    StoreContext,
    control::{
        ControlService,
        HostDoc,
        UnhostDoc,
        authenticate,
    },
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

    ctx.db
        .call_mut({
            let did_str = did_str.clone();
            move |conn| {
                ensure_quota_exists(conn, &did_str)?;
                conn.execute(
                    "INSERT OR IGNORE INTO hosted_docs (ns, owner, bytes_used) VALUES (?, ?, 0)",
                    params![ns_str, did_str],
                )?;
                Ok(())
            }
        })
        .await?;

    let doc = crate::docs::ensure_open(&ctx.docs, inner.ns).await?;

    // Agreeing to host means agreeing to replicate. Importing alone leaves the
    // namespace out of the sync set, where the host rejects every incoming
    // request with `NotFound` — availability being the entire point of hosting.
    doc.start_sync(Vec::new()).await?;

    n0_future::task::spawn(meter_doc(Arc::clone(&ctx), did, inner.ns, doc));

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

    ctx.db
        .call_mut({
            let did_str = did_str.clone();
            move |conn| {
                let bytes_used: i64 = conn
                    .query_row(
                        "SELECT bytes_used FROM hosted_docs WHERE owner = ? AND ns = ?",
                        params![did_str, ns_str],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                conn.execute(
                    "DELETE FROM hosted_docs WHERE owner = ? AND ns = ?",
                    params![did_str, ns_str],
                )?;
                if bytes_used > 0 {
                    release_bytes(conn, &did_str, bytes_used)?;
                }
                Ok(())
            }
        })
        .await?;

    ctx.docs.api().drop_doc(inner.ns).await?;

    tx.send(Ok(())).await?;
    Ok(())
}

/// Meters a hosted doc against its owner's quota: charges the content size of
/// entries already present, then charges each live insert. On quota overflow it
/// logs and stops metering (pause/unhost is a follow-up); it never rejects an
/// individual write it cannot interpret.
async fn meter_doc(ctx: Arc<StoreContext>, owner: Did, ns: NamespaceId, doc: Doc) {
    if let Err(err) = meter_existing(&ctx, &owner, ns, &doc).await {
        tracing::warn!(%ns, "failed metering existing doc entries: {err}");
    }

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

        if let Err(err) = charge(&ctx, &owner, ns, i64::try_from(len).unwrap_or(i64::MAX)).await {
            tracing::warn!(%ns, %owner, "hosted doc over quota, metering stopped: {err}");
            return;
        }
    }
}

async fn meter_existing(
    ctx: &Arc<StoreContext>,
    owner: &Did,
    ns: NamespaceId,
    doc: &Doc,
) -> anyhow::Result<()> {
    let mut total: i64 = 0;
    let entries = doc.get_many(Query::all()).await?;
    let mut entries = std::pin::pin!(entries);
    while let Some(entry) = entries.next().await {
        total += i64::try_from(entry?.content_len()).unwrap_or(i64::MAX);
    }
    if total > 0 {
        charge(ctx, owner, ns, total).await?;
    }
    Ok(())
}

/// Reserves `len` bytes against the owner and adds them to the doc's ledger in
/// a single transaction.
async fn charge(
    ctx: &Arc<StoreContext>,
    owner: &Did,
    ns: NamespaceId,
    len: i64,
) -> anyhow::Result<()> {
    let owner = owner.to_string();
    let ns = ns.to_string();
    ctx.db
        .call_mut(move |conn| {
            let tx = conn.transaction()?;
            if reserve_bytes(&tx, &owner, len).is_err() {
                anyhow::bail!("quota exceeded");
            }
            tx.execute(
                "UPDATE hosted_docs SET bytes_used = bytes_used + ? WHERE owner = ? AND ns = ?",
                params![len, owner, ns],
            )?;
            tx.commit()?;
            Ok(())
        })
        .await
}
