use std::collections::BTreeSet;

use bevy_wds::registry_clients;
use iroh::{
    EndpointId,
    PublicKey,
};
use iroh_docs::NamespaceId;
use tracing::{
    info,
    warn,
};

use crate::gossip::GossipCtx;

/// Discovers peers to bootstrap gossip against by asking each followed registry
/// who is currently in `space`.
///
/// Presence is queried rather than synced: it is soft state with a short TTL,
/// and every returned record is verified against its announcer's DID before it
/// is trusted enough to dial.
pub async fn find_bootstrap_peers(
    ctx: &GossipCtx,
    space: NamespaceId,
) -> anyhow::Result<BTreeSet<PublicKey>> {
    let mut bootstrap = BTreeSet::new();

    for registry in registry_clients() {
        let occupants = match registry.occupants(space).await {
            Ok(occupants) => occupants,
            Err(err) => {
                warn!(?err, "Failed querying registry presence");
                continue;
            }
        };

        let mut listed = Vec::new();
        for presence in occupants {
            let Ok(endpoint) = EndpointId::from_bytes(&presence.endpoint) else {
                continue;
            };
            listed.push(endpoint.fmt_short().to_string());
            if endpoint == ctx.endpoint.id() {
                continue;
            }
            bootstrap.insert(endpoint);
        }

        // A registry holds presence in memory with a short TTL, so this list
        // routinely contains endpoints from processes that have already exited
        // — dialable in principle, answering nothing in practice.
        info!(
            me = %ctx.endpoint.id().fmt_short(),
            occupants = ?listed,
            "Registry presence for space"
        );
    }

    Ok(bootstrap)
}
