//! Which registries this node follows, resolved from their DIDs.

use std::{
    collections::BTreeSet,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use iroh::{
    Endpoint,
    EndpointAddr,
    EndpointId,
};
use iroh_docs::NamespaceId;
use parking_lot::RwLock;
use tracing::{
    info,
    warn,
};
use unavi_identity::{
    auth::EndpointAuth,
    did_document,
    identity::Identity,
    resolver::Resolver,
};
use unavi_store::Store;
use xdid::core::did::Did;

use crate::client::RegistryClient;

/// How long a sync pass waits for a registry's proof before leaving it for a
/// later pass. The first dial to a cold node spends its time on discovery, so
/// this is generous; a pass that gives up is retried, not lost.
const PROVE_WAIT: Duration = Duration::from_secs(30);

/// A registry to follow: the DID it is known by and the endpoint that DID
/// names.
#[derive(Debug, Clone)]
pub struct Target {
    pub did:  Did,
    pub addr: EndpointAddr,
}

/// A target that came out of a [`Followed::sync`] pass unfollowed.
#[derive(Debug, Clone)]
pub enum NotFollowed {
    /// The registry was unreachable, or proved nothing this pass. The next
    /// pass may well succeed: a cold start, a restarting node or a peer that
    /// was still being discovered all land here.
    Unproven(Target),
    /// The endpoint proved a DID other than the one it was resolved from, so
    /// its DID document points this node at someone else's registry. No
    /// retry makes that hold.
    WrongDid(Target),
}

impl NotFollowed {
    #[must_use]
    pub const fn target(&self) -> &Target {
        match self {
            Self::Unproven(target) | Self::WrongDid(target) => target,
        }
    }

    /// The target to try again, or `None` if trying again cannot help.
    #[must_use]
    pub fn retryable(&self) -> Option<&Target> {
        matches!(self, Self::Unproven(_)).then(|| self.target())
    }
}

/// The registries this node follows, and the view docs each publishes. Clones
/// share one set.
///
/// Empty until [`Self::sync`] first completes.
#[derive(Clone, Default)]
pub struct Followed(Arc<RwLock<Vec<FollowedRegistry>>>);

struct FollowedRegistry {
    client: RegistryClient,
    views:  Vec<NamespaceId>,
}

impl Followed {
    /// The view docs of every followed registry.
    #[must_use]
    pub fn views(&self) -> Vec<NamespaceId> {
        self.0
            .read()
            .iter()
            .flat_map(|registry| registry.views.iter().copied())
            .collect()
    }

    #[must_use]
    pub fn is_view(&self, ns: NamespaceId) -> bool {
        self.0
            .read()
            .iter()
            .any(|registry| registry.views.contains(&ns))
    }

    #[must_use]
    pub fn clients(&self) -> Vec<RegistryClient> {
        self.0
            .read()
            .iter()
            .map(|registry| registry.client.clone())
            .collect()
    }

    /// Builds a client per target, syncs each one's views, and adds those that
    /// proved their DID to the followed set.
    ///
    /// A registry that answered in an earlier pass stays followed through one
    /// that does not: losing presence discovery over a slow handshake is worse
    /// than trusting a proof made minutes ago. What is dropped outright is a
    /// registry that proved a different DID than its document names.
    ///
    /// Returns the targets that were not followed, so the caller can retry
    /// them rather than run unfollowed.
    pub async fn sync(
        &self,
        store: &Store,
        endpoint: &Endpoint,
        auth: &EndpointAuth,
        targets: &[Target],
        identity: &Arc<Identity>,
        resolver: &Arc<Resolver>,
    ) -> Vec<NotFollowed> {
        let mut followed = Vec::new();
        let mut unfollowed = Vec::new();

        for target in targets {
            let client = RegistryClient::new(
                endpoint,
                target.addr.clone(),
                Arc::clone(identity),
                Arc::clone(resolver),
            );

            let synced = match client.sync_views(store).await {
                Ok(synced) => synced,
                Err(err) => {
                    warn!(did = %target.did, ?err, "failed syncing registry views");
                    unfollowed.push(NotFollowed::Unproven(target.clone()));
                    continue;
                }
            };

            match auth.prove(target.addr.id, PROVE_WAIT).await {
                Some(did) if did == target.did => {
                    followed.push(FollowedRegistry {
                        client,
                        views: synced,
                    });
                }
                Some(did) => {
                    warn!(
                        did = %target.did,
                        proved = %did,
                        peer = %target.addr.id.fmt_short(),
                        "registry proved another DID; not following"
                    );
                    unfollowed.push(NotFollowed::WrongDid(target.clone()));
                }
                None => {
                    warn!(
                        did = %target.did,
                        peer = %target.addr.id.fmt_short(),
                        "registry did not prove its DID; retrying"
                    );
                    unfollowed.push(NotFollowed::Unproven(target.clone()));
                }
            }
        }

        let wrong: BTreeSet<EndpointId> = unfollowed
            .iter()
            .filter(|not| matches!(not, NotFollowed::WrongDid(_)))
            .map(|not| not.target().addr.id)
            .collect();

        let mut followed_now = self.0.write();
        followed_now.retain(|registry| !wrong.contains(&registry.client.host().id));
        for registry in followed {
            let host = registry.client.host().id;
            followed_now.retain(|followed| followed.client.host().id != host);
            followed_now.push(registry);
        }

        unfollowed
    }
}

/// Resolves every target, returning the ones that answered alongside the DIDs
/// that did not.
///
/// An unresolved target is worth retrying rather than failing the load over. A
/// app with no reachable registry still runs peer to peer.
pub async fn resolve_batch(dids: Vec<String>, resolver: &Resolver) -> (Vec<Target>, Vec<String>) {
    let mut targets = Vec::new();
    let mut unresolved = Vec::new();

    for did in dids {
        match resolve_target(&did, resolver).await {
            Ok(target) => {
                info!(%did, "following registry");
                targets.push(target);
            }
            Err(err) => {
                warn!(%did, ?err, "failed to resolve registry");
                unresolved.push(did);
            }
        }
    }

    (targets, unresolved)
}

async fn resolve_target(did: &str, resolver: &Resolver) -> anyhow::Result<Target> {
    let did = Did::from_str(did)?;
    let doc = resolver.resolve(&did).await?;
    let endpoint = did_document::endpoint_of(&doc)
        .ok_or_else(|| anyhow::anyhow!("DID document names no iroh endpoint"))?;

    Ok(Target {
        did,
        addr: EndpointAddr::from(endpoint),
    })
}
