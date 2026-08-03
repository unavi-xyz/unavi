//! Client-side registry access: sync a host's registry doc and read the
//! signed beacons in it, verifying each announcer's signature.

use futures::StreamExt;
use iroh::EndpointAddr;
use iroh_blobs::api::blobs::Blobs;
use iroh_docs::{
    Capability,
    NamespaceId,
    protocol::Docs,
    store::Query,
};
use time::OffsetDateTime;

use crate::{
    actor::Actor,
    format::{
        Beacon,
        keys,
    },
    signed_bytes::{
        SignedBytes,
        verify_did_signature,
    },
};

/// Discovers populated spaces by syncing each host's registry doc and reading
/// its verified beacons.
pub async fn discover(actors: &[Actor], docs: &Docs, blobs: &Blobs) -> Vec<Beacon> {
    let mut out = Vec::new();
    for actor in actors {
        let Ok(Some(registry_ns)) = actor.registry_id().await else {
            continue;
        };
        let _ = sync_registry(docs, registry_ns, actor.host().clone()).await;
        if let Ok(mut beacons) = read_verified_beacons(docs, blobs, registry_ns).await {
            out.append(&mut beacons);
        }
    }
    out
}

/// Imports (read-only) and starts syncing a registry doc from `host`.
pub async fn sync_registry(docs: &Docs, ns: NamespaceId, host: EndpointAddr) -> anyhow::Result<()> {
    let doc = match docs.api().open(ns).await? {
        Some(doc) => doc,
        None => docs.api().import_namespace(Capability::Read(ns)).await?,
    };
    doc.start_sync(vec![host]).await?;
    Ok(())
}

/// Reads all unexpired, signature-verified beacons from a synced registry doc.
pub async fn read_verified_beacons(
    docs: &Docs,
    blobs: &Blobs,
    ns: NamespaceId,
) -> anyhow::Result<Vec<Beacon>> {
    let Some(doc) = docs.api().open(ns).await? else {
        return Ok(Vec::new());
    };

    let query = Query::single_latest_per_key().key_prefix(keys::BEACONS_PREFIX);
    let entries = doc.get_many(query).await?;
    let mut entries = std::pin::pin!(entries);

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let mut out = Vec::new();

    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let Ok(bytes) = blobs.get_bytes(entry.content_hash()).await else {
            continue;
        };
        let Ok(signed) = postcard::from_bytes::<SignedBytes<Beacon>>(&bytes) else {
            continue;
        };
        let Ok(beacon) = signed.payload() else {
            continue;
        };
        if now >= beacon.expires {
            continue;
        }
        if !verify_did_signature(&signed, &beacon.did).await {
            continue;
        }
        out.push(beacon);
    }

    Ok(out)
}
