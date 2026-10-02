//! Multiplayer for spaces: finding peers, the `wired/space/1` link, and the
//! pin/hold/session state replicated over it.
//!
//! Who may state what is decided by authorship: a document's author is the
//! DID holding its namespace write key, and only it pins the document, holds
//! it by default, and writes its session state.

use std::time::Duration;

use bevy::{
    app::AnimationSystems,
    prelude::*,
    time::common_conditions::on_timer,
};
use unavi_policy::trust::TrustTable;
use unavi_portal::{
    crossing::apply_crossings,
    echo::maintain_echoes,
};

pub mod authority;
pub mod avatar_sync;
pub mod discovery;
pub mod grid;
pub mod identity;
pub mod index;
pub mod link;
pub mod membership;
pub mod moderation;
pub mod object_sync;
pub mod pinned;
pub mod portal;
pub mod replication;
mod scene;
pub mod spawn;
pub mod travel;

pub struct SpacePlugin {
    /// Where the trust table persists. `None` leaves blocks effective for the
    /// session only.
    pub storage: Option<unavi_local::DeviceStorage>,
}

const SEND_INTERVAL_UPDATE: Duration = Duration::from_secs(5);

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<membership::MembershipPlugin>() {
            app.add_plugins(membership::MembershipPlugin);
        }

        let storage = self.storage.clone().unwrap_or_default();
        let trust = TrustTable::load(storage.clone()).unwrap_or_else(|err| {
            error!(
                ?err,
                "Trust table could not be read; every block is inactive this session"
            );
            TrustTable::new(storage)
        });
        app.insert_resource(trust);

        replication::guards::init_indexes(app.world_mut());
        app.init_resource::<index::Index<discovery::Peer>>()
            .init_resource::<index::Index<avatar_sync::RemoteAgent>>()
            .init_resource::<grid::SpaceGrid>()
            .init_resource::<grid::ActiveSpace>()
            .init_resource::<travel::PendingTravel>()
            .init_resource::<replication::Replicas>()
            .init_resource::<discovery::HeardPresence>()
            .init_resource::<discovery::PeerPresence>()
            .init_resource::<discovery::gossip::ActiveSpaceSignal>()
            .add_observer(grid::assign_anchor)
            .add_observer(grid::promote_first_space)
            .add_observer(grid::release_anchor)
            .add_observer(authority::reassign_doc_quota)
            .add_observer(authority::forget_peer_quota)
            .add_observer(authority::record_minted)
            .add_observer(authority::forget_minted)
            .add_observer(link::register_protocol)
            .add_observer(link::disconnect_peer)
            .add_observer(discovery::gossip::leave_space_topic)
            .add_observer(portal::spawn_portal_space)
            .add_observer(scene::despawn_space_scene)
            .add_observer(scene::spawn_space_scene)
            .add_observer(pinned::adopt_pinned_docs)
            .add_systems(
                PostUpdate,
                (grid::recenter_active_space, grid::apply_anchor_offsets)
                    .chain()
                    .after(apply_crossings)
                    .before(maintain_echoes)
                    .before(TransformSystems::Propagate),
            )
            .add_systems(
                PostUpdate,
                avatar_sync::receive::apply_remote_bones
                    .after(AnimationSystems)
                    .before(TransformSystems::Propagate),
            )
            .add_systems(
                FixedUpdate,
                (
                    (discovery::track_peers, link::dial::dial_peers).chain(),
                    discovery::publish_blob_providers,
                    discovery::registry::announce_presence,
                    discovery::gossip::adopt_gossip,
                    discovery::gossip::join_space_topics,
                    avatar_sync::send::send_agent_pose,
                    avatar_sync::send::set_agent_intervals.run_if(on_timer(SEND_INTERVAL_UPDATE)),
                    object_sync::systems::send_object_poses,
                    object_sync::systems::reconcile_object_holds,
                    (
                        object_sync::systems::apply_remote_objects,
                        object_sync::systems::advance_object_interp,
                    )
                        .chain(),
                    portal::enter_peeked_space,
                    scene::start_space_fetch,
                    scene::instance_pending_scenes,
                    pinned::fetch_pinned_docs,
                    pinned::instance_pinned_docs,
                    pinned::prune_pinned_docs,
                ),
            )
            .add_systems(
                Update,
                portal::sync_portal_home.before(unavi_portal::destination::resolve_destinations),
            )
            .add_systems(
                Update,
                (
                    discovery::gossip::publish_active_space,
                    avatar_sync::receive::apply_remote_poses,
                    avatar_sync::receive::advance_remote_lerp,
                )
                    .chain(),
            );
    }
}
