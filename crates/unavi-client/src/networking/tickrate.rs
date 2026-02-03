//! Distance-based tickrate adjustment for outbound player data.

use bevy::prelude::*;
use unavi_player::{LocalPlayer, PlayerEntities, PlayerRig};

use crate::networking::{
    player_receive::RemotePlayer,
    thread::{
        NetworkCommand, NetworkingThread,
        space::{MAX_TICKRATE, MIN_TICKRATE},
    },
};

/// Distance at which tickrate reaches minimum (in meters).
const MAX_DISTANCE: f32 = 64.0;

/// Per-player tickrate configuration.
#[derive(Component, Clone)]
pub struct PlayerTickrateConfig {
    pub min: u8,
    pub max: u8,
}

impl Default for PlayerTickrateConfig {
    fn default() -> Self {
        Self {
            min: MIN_TICKRATE,
            max: MAX_TICKRATE,
        }
    }
}

/// Calculates tickrate based on distance.
/// Returns max tickrate at distance 0, min tickrate at [`MAX_DISTANCE`].
#[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
fn tickrate_for_distance(distance: f32, min: u8, max: u8) -> u8 {
    let t = (distance / MAX_DISTANCE).clamp(0.0, 1.0);
    let rate = f32::from(max) - t * f32::from(max - min);
    rate.round() as u8
}

/// Updates peer tickrates based on distance from local player.
/// Runs on a timer to avoid excessive updates.
pub fn update_peer_tickrates(
    nt: Res<NetworkingThread>,
    local_player: Query<&PlayerEntities, With<LocalPlayer>>,
    body_transforms: Query<&GlobalTransform, With<PlayerRig>>,
    remote_players: Query<(&RemotePlayer, &Transform, &PlayerTickrateConfig)>,
) {
    let Ok(entities) = local_player.single() else {
        return;
    };
    let Ok(local_tr) = body_transforms.get(entities.body) else {
        return;
    };
    let local_pos = local_tr.translation();

    for (remote, remote_tr, config) in &remote_players {
        let distance = local_pos.distance(remote_tr.translation);
        let tickrate = tickrate_for_distance(distance, config.min, config.max);

        let _ = nt.command_tx.try_send(NetworkCommand::SetPeerTickrate {
            peer: remote.0,
            tickrate,
        });
    }
}
