use std::collections::BTreeSet;

use iroh::{
    EndpointId,
    PublicKey,
};
use iroh_docs::NamespaceId;
use tracing::warn;

use crate::gossip::GossipCtx;

/// Discovers peers hosting `space` by syncing each home server's registry doc
/// and reading its signature-verified beacons.
pub async fn find_bootstrap_peers(
    ctx: &GossipCtx,
    space: NamespaceId,
) -> anyhow::Result<BTreeSet<PublicKey>> {
    let mut bootstrap = BTreeSet::new();

    let target_actors = if ctx.sync_targets.is_empty() {
        vec![ctx.actor.clone()]
    } else {
        ctx.sync_targets.clone()
    };

    for actor in target_actors {
        let Some(registry_ns) = actor.registry_id().await? else {
            continue;
        };

        if let Err(err) =
            wds::registry::sync_registry(&ctx.docs, registry_ns, actor.host().clone()).await
        {
            warn!(?err, "Failed to sync registry doc");
            continue;
        }

        let beacons = wds::registry::read_verified_beacons(&ctx.docs, &ctx.blobs, registry_ns)
            .await
            .unwrap_or_default();

        for beacon in beacons {
            if beacon.space != space {
                continue;
            }
            let Ok(endpoint) = EndpointId::from_bytes(&beacon.endpoint) else {
                continue;
            };
            if endpoint == ctx.endpoint.id() {
                continue;
            }
            bootstrap.insert(endpoint);
        }
    }

    Ok(bootstrap)
}
