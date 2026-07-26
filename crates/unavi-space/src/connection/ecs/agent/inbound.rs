use std::{
    sync::{
        LazyLock,
        Mutex,
    },
    time::{
        Duration,
        Instant,
    },
};

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use blake3::Hash;
use iroh::EndpointId;
use unavi_avatar::Avatar;
use unavi_manifold::EchoBody;

use crate::{
    Space,
    peer::{
        ActiveSpaces,
        Peer,
    },
};

const MIN_LERP: Duration = Duration::from_millis(50);
const MAX_LERP: Duration = Duration::from_millis(500);

pub struct ResolvedPose {
    pub space: Hash,
    pub root:  Transform,
}

static POSE_INBOX: LazyLock<Mutex<HashMap<EndpointId, (Instant, ResolvedPose)>>> =
    LazyLock::new(|| Mutex::new(HashMap::default()));

pub fn submit_pose(peer: EndpointId, pose: ResolvedPose) {
    POSE_INBOX
        .lock()
        .expect("pose inbox")
        .insert(peer, (Instant::now(), pose));
}

#[derive(Component)]
pub struct RemoteAgent(pub EndpointId);

#[derive(Component)]
pub struct PoseLerp {
    prev:      Transform,
    target:    Transform,
    elapsed:   Duration,
    duration:  Duration,
    last_recv: Instant,
}

impl PoseLerp {
    const fn snapped(root: Transform, recv: Instant) -> Self {
        Self {
            prev:      root,
            target:    root,
            elapsed:   Duration::ZERO,
            duration:  MIN_LERP,
            last_recv: recv,
        }
    }

    fn retarget(&mut self, current: Transform, target: Transform, recv: Instant) {
        self.prev = current;
        self.target = target;
        self.elapsed = Duration::ZERO;
        self.duration = recv
            .saturating_duration_since(self.last_recv)
            .clamp(MIN_LERP, MAX_LERP);
        self.last_recv = recv;
    }
}

pub fn apply_remote_poses(
    time: Res<Time>,
    spaces: Query<(Entity, &Space)>,
    mut peers: Query<(&Peer, &mut ActiveSpaces)>,
    mut remotes: Query<(Entity, &RemoteAgent, &ChildOf, &Transform, &mut PoseLerp)>,
    mut commands: Commands,
    mut warned: Local<bevy::platform::collections::HashSet<Hash>>,
) {
    let updates = std::mem::take(&mut *POSE_INBOX.lock().expect("pose inbox"));
    let now = time.elapsed_secs();

    for (peer, (recv, resolved)) in updates {
        // The live pose stream is the authoritative, low-latency signal for
        // which space a peer is in; refresh presence from it so a connected
        // peer never expires from stale gossip (gossip is discovery only).
        if let Some((_, mut active_spaces)) = peers.iter_mut().find(|(p, _)| p.0.id == peer) {
            active_spaces.0.insert(resolved.space, now);
        }

        let Some((space, _)) = spaces.iter().find(|(_, s)| s.0 == resolved.space) else {
            // The peer moved into a space we have not instanced locally, so they
            // have left our view; despawn any avatar rather than leaving it
            // frozen at its last known position.
            if let Some((entity, ..)) = remotes.iter().find(|(_, r, ..)| r.0 == peer) {
                commands.entity(entity).despawn();
            }
            if warned.insert(resolved.space) {
                warn!(
                    peer = %peer,
                    space = %resolved.space,
                    local = ?spaces.iter().map(|(_, s)| s.0).collect::<Vec<_>>(),
                    "Dropping remote pose: peer's space is not instanced locally",
                );
            }
            continue;
        };
        warned.remove(&resolved.space);

        let Some((entity, _, child_of, current, mut lerp)) =
            remotes.iter_mut().find(|(_, r, ..)| r.0 == peer)
        else {
            info!(peer = %peer, space = %resolved.space, pos = ?resolved.root.translation, "Instancing remote agent");
            commands.spawn((
                RemoteAgent(peer),
                Avatar,
                EchoBody,
                resolved.root,
                PoseLerp::snapped(resolved.root, recv),
                ChildOf(space),
            ));
            continue;
        };

        if child_of.parent() == space {
            lerp.retarget(*current, resolved.root, recv);
        } else {
            // Space changed: reparent under the new anchor and snap rather than
            // interpolating across a grid jump.
            commands.entity(entity).insert(ChildOf(space));
            *lerp = PoseLerp::snapped(resolved.root, recv);
        }
    }
}

pub fn advance_remote_lerp(time: Res<Time>, mut remotes: Query<(&mut PoseLerp, &mut Transform)>) {
    let dt = time.delta();

    for (mut lerp, mut transform) in &mut remotes {
        lerp.elapsed = (lerp.elapsed + dt).min(lerp.duration);

        let t = if lerp.duration.is_zero() {
            1.0
        } else {
            lerp.elapsed.as_secs_f32() / lerp.duration.as_secs_f32()
        };

        transform.translation = lerp.prev.translation.lerp(lerp.target.translation, t);
        transform.rotation = lerp.prev.rotation.slerp(lerp.target.rotation, t);
    }
}

pub fn despawn_remote_agent(
    trigger: On<Remove, Peer>,
    peers: Query<&Peer>,
    remotes: Query<(Entity, &RemoteAgent)>,
    mut commands: Commands,
) {
    let Ok(peer) = peers.get(trigger.entity) else {
        return;
    };

    for (entity, remote) in remotes {
        if remote.0 == peer.0.id {
            commands.entity(entity).despawn();
        }
    }
}
