use std::time::Duration;

use bevy::prelude::*;

use crate::{
    networking::thread::{NetworkCommand, NetworkingThread},
    space::Space,
};

const BEACON_TTL: Duration = Duration::from_mins(1);

pub fn publish_beacons(
    nt: Res<NetworkingThread>,
    time: Res<Time>,
    spaces: Query<&Space>,
    mut last: Local<Duration>,
) {
    if spaces.is_empty() {
        return;
    }

    let now = time.elapsed();
    if !last.is_zero() && now.saturating_sub(*last) < BEACON_TTL {
        return;
    }
    *last = now;

    for space in spaces {
        if let Err(e) = nt.command_tx.send(NetworkCommand::PublishBeacon {
            id: space.0,
            ttl: BEACON_TTL,
        }) {
            error!("error sending network command: {e:?}");
        }
    }
}
