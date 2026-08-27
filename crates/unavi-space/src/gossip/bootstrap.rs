use std::collections::BTreeSet;

use iroh::PublicKey;
use iroh_docs::NamespaceId;
use tracing::{
    info,
    warn,
};
use unavi_registry::follow::registry_clients;

use crate::gossip::GossipCtx;

/// Discovers peers to bootstrap gossip against by asking each followed registry
/// who is currently in `space`. Presence records are soft state with a short
/// TTL, verified against the announcer's DID before dialing.
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
            let endpoint = presence.endpoint;
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
