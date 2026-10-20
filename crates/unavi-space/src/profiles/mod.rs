//! Other users' profiles, asked of their peers and read as their documents
//! sync.

use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};

use bevy::{
    prelude::*,
    time::common_conditions::on_timer,
};
use bevy_async::task;
use bevy_iroh::{
    endpoint::IrohEndpoint,
    store::DataStore,
};
use iroh::{
    Endpoint,
    EndpointAddr,
};
use n0_future::StreamExt;
use parking_lot::RwLock;
use smol_str::SmolStr;
use unavi_identity::{
    authorship,
    profile::protocol,
    resolver::Resolver,
};
use unavi_policy::trust::Trust;
use unavi_store::{
    Document,
    Events,
    Store,
};
use xdid::core::did::Did;

use crate::{
    authority::SpaceView,
    avatar_sync::RemoteAgent,
    discovery::Peer,
    index::Index,
};

/// How long a DID whose peer gave no usable answer waits before it is asked
/// again.
const RETRY: Duration = Duration::from_secs(60);
/// How long joining a profile may take before the ask counts as failed.
const JOIN_DEADLINE: Duration = Duration::from_secs(30);
/// How often co-present peers are checked for profiles not yet asked for.
const ASK_INTERVAL: Duration = Duration::from_secs(1);

pub struct ProfilesPlugin;

impl Plugin for ProfilesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Profiles>()
            .add_systems(FixedUpdate, ask_profiles.run_if(on_timer(ASK_INTERVAL)));
    }
}

/// What a user's profile says about them. Self-asserted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Profile {
    pub name:        Option<SmolStr>,
    pub description: Option<SmolStr>,
}

enum Lookup {
    Asking,
    /// The profile document is joined. Its content is `None` until the meta
    /// has arrived.
    Found(Option<Profile>),
    Failed {
        at: Duration,
    },
}

/// The profiles of the users this peer has met, by DID.
#[derive(Resource, Clone, Default)]
pub struct Profiles(Arc<RwLock<HashMap<Did, Lookup>>>);

impl Profiles {
    /// `did`'s profile, once its meta has arrived.
    #[must_use]
    pub fn get(&self, did: &Did) -> Option<Profile> {
        match self.0.read().get(did)? {
            Lookup::Found(profile) => profile.clone(),
            Lookup::Asking | Lookup::Failed { .. } => None,
        }
    }

    /// Marks `did` as being asked for. Whether the caller should ask: `false`
    /// while an ask is in flight, once one has succeeded, and within
    /// [`RETRY`] of a failure.
    fn begin(&self, did: &Did, now: Duration) -> bool {
        let mut lookups = self.0.write();
        let ask = match lookups.get(did) {
            None => true,
            Some(Lookup::Failed { at }) => now.saturating_sub(*at) >= RETRY,
            Some(Lookup::Asking | Lookup::Found(_)) => false,
        };
        if ask {
            lookups.insert(did.clone(), Lookup::Asking);
        }
        ask
    }

    fn set(&self, did: &Did, lookup: Lookup) {
        self.0.write().insert(did.clone(), lookup);
    }
}

/// Asks each co-present peer for its user's profile, once per DID.
pub fn ask_profiles(
    time: Res<Time>,
    agents: Query<&RemoteAgent>,
    peers: Query<&Peer>,
    peer_index: Res<Index<Peer>>,
    endpoints: Query<&IrohEndpoint>,
    view: Option<Res<SpaceView>>,
    store: Option<Res<DataStore>>,
    profiles: Res<Profiles>,
) {
    let (Some(view), Some(store)) = (view, store) else {
        return;
    };
    let Some(endpoint) = endpoints.iter().next() else {
        return;
    };
    let identity = view.identity();
    let now = time.elapsed();
    for agent in &agents {
        let Some(did) = identity.bindings.did_of(agent.0) else {
            continue;
        };
        if view.trust_of_did(&did) == Trust::Blocked || !profiles.begin(&did, now) {
            continue;
        }
        let addr = peer_index
            .get(agent.0)
            .and_then(|entity| peers.get(entity).ok())
            .map_or_else(|| EndpointAddr::from(agent.0), |peer| peer.0.clone());
        let ask = Ask {
            endpoint: endpoint.0.clone(),
            store:    store.0.clone(),
            resolver: Arc::clone(&identity.resolver),
            profiles: profiles.clone(),
        };
        task::spawn(ask.run(did, addr, now));
    }
}

struct Ask {
    endpoint: Endpoint,
    store:    Store,
    resolver: Arc<Resolver>,
    profiles: Profiles,
}

impl Ask {
    async fn run(self, did: Did, addr: EndpointAddr, now: Duration) {
        let peer = addr.id;
        let ns = match protocol::ask(&self.endpoint, peer, &did, &self.resolver).await {
            Ok(ns) => ns,
            Err(err) => {
                debug!(%did, %peer, ?err, "no profile from peer");
                self.profiles.set(&did, Lookup::Failed { at: now });
                return;
            }
        };
        let (doc, events) =
            match n0_future::time::timeout(JOIN_DEADLINE, self.store.join(ns, vec![addr])).await {
                Ok(Ok(joined)) => joined,
                Ok(Err(err)) => {
                    warn!(%did, %ns, ?err, "could not join a profile");
                    self.profiles.set(&did, Lookup::Failed { at: now });
                    return;
                }
                Err(_) => {
                    warn!(%did, %ns, "joining a profile timed out");
                    self.profiles.set(&did, Lookup::Failed { at: now });
                    return;
                }
            };
        self.profiles.set(&did, Lookup::Found(None));
        self.follow(&did, &doc, events).await;
        // Asked again once `RETRY` has passed since this ask began.
        self.profiles.set(&did, Lookup::Failed { at: now });
    }

    /// Rereads the profile whenever its document changes, until the
    /// subscription ends.
    async fn follow(&self, did: &Did, doc: &Document, mut events: Events) {
        loop {
            match authorship::recorded_meta(doc).await {
                Ok(meta) => {
                    let profile = meta.map(|meta| Profile {
                        name:        meta.name,
                        description: meta.description,
                    });
                    self.profiles.set(did, Lookup::Found(profile));
                }
                Err(err) => debug!(%did, ?err, "unreadable profile meta"),
            }
            match events.next().await {
                Some(Ok(_)) => {}
                Some(Err(err)) => {
                    debug!(%did, ?err, "profile subscription failed");
                    return;
                }
                None => return,
            }
        }
    }
}

#[cfg(test)] mod tests;
