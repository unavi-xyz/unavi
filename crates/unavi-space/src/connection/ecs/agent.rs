use std::time::Duration;

use bevy::prelude::*;
use unavi_agent::LocalAgent;

use crate::{
    connection::{
        ecs::{LastTick, PeerStream, Tickrate},
        types::{IFrame, pose::Pose},
    },
    peer::Peer,
};

#[derive(Component)]
#[require(Tickrate)]
pub struct AgentSender(pub async_channel::Sender<Pose<IFrame>>);

pub fn send_agent_pose(
    time: Res<Time>,
    agent: Query<&Transform, With<LocalAgent>>,
    mut streams: Query<(&AgentSender, &Tickrate, &mut LastTick)>,
) {
    let Ok(root) = agent.single() else {
        return;
    };

    let pose = Pose {
        root: root.into(),
        ..Default::default()
    };

    let now = time.elapsed();

    streams
        .par_iter_mut()
        .for_each(|(sender, tickrate, mut last_tick)| {
            if sender.0.is_full() {
                return;
            }

            if last_tick.0 + tickrate.0 > now {
                return;
            }

            last_tick.0 = now;

            if let Err(err) = sender.0.try_send(pose.clone()) {
                error!(?err, "Send error");
            }
        });
}

const MAX_TICKRATE: Duration = Duration::from_millis(200);
const MIN_TICKRATE: Duration = Duration::from_millis(50);

const MAX_DIST: f32 = 50.0;
const MIN_DIST: f32 = 4.0;

pub fn set_agent_tickrates(
    agent: Query<&Transform, With<LocalAgent>>,
    peers: Query<(&Peer, &Transform), Without<LocalAgent>>,
    streams: Query<(&PeerStream, &mut Tickrate)>,
) {
    let Ok(root) = agent.single() else {
        return;
    };

    // Measure distance to each peer.
    for (target, mut tickrate) in streams {
        let Some((_, transform)) = peers.iter().find(|(p, _)| p.0.id == target.0) else {
            continue;
        };

        let dist = root.translation.distance(transform.translation);
        let dist = dist.abs().clamp(MIN_DIST, MAX_DIST);
        let s = (dist - MIN_DIST) / (MAX_DIST - MIN_DIST);
        let secs = MIN_TICKRATE
            .as_secs_f32()
            .lerp(MAX_TICKRATE.as_secs_f32(), s);
        tickrate.0 = Duration::from_secs_f32(secs);
    }
}
