use bevy::prelude::*;
use xdid::core::did_url::DidUrl;

use unavi_server_service::from_server::ControlMessage;

use crate::{
    auth::LocalActor,
    space::networking::{
        connection::state::{ConnectionAttempt, ConnectionState},
        streams::{
            TransformChannels,
            publish::{PublishInterval, TransformPublishState},
        },
        thread::{NetworkCommand, NetworkingThread},
    },
};

mod create;
pub mod networking;
pub mod record_ref_url;
mod tickrate;

pub struct SpacePlugin;

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(networking::NetworkingPlugin)
            .add_systems(Update, send_actor_to_networking_thread)
            .add_systems(FixedUpdate, tickrate::set_space_tickrates);
    }
}

fn send_actor_to_networking_thread(actor: Res<LocalActor>, networking: Res<NetworkingThread>) {
    if !actor.is_changed() {
        return;
    }

    let Some(actor_val) = actor.0.clone() else {
        return;
    };

    let command = NetworkCommand::SetActor { actor: actor_val };

    if let Err(e) = networking.command_tx.send(command) {
        error!("Failed to send SetActor command: {e:?}");
    }
}

/// Host server connection entity.
#[derive(Component)]
#[require(HostPlayers)]
pub struct Host {
    pub connect_url: String,
}

#[derive(Component)]
pub struct HostTransformChannels {
    pub players: TransformChannels,
}

#[derive(Component)]
pub struct HostControlChannel {
    pub _tx: flume::Sender<ControlMessage>,
    pub rx: flume::Receiver<ControlMessage>,
}

#[derive(Component, Default)]
#[relationship_target(relationship = PlayerHost)]
pub struct HostPlayers(Vec<Entity>);

/// Remote player entity.
#[derive(Component)]
pub struct RemotePlayer {
    pub player_id: u64,
}

#[derive(Component)]
#[relationship(relationship_target = HostPlayers)]
pub struct PlayerHost(Entity);

/// Declarative space definition.
/// Upon add, connection state components are inserted and the join process begins.
#[derive(Component)]
#[require(
    ConnectionState,
    ConnectionAttempt,
    PublishInterval,
    TransformPublishState
)]
pub struct Space {
    pub url: DidUrl,
}

impl Space {
    pub const fn new(url: DidUrl) -> Self {
        Self { url }
    }
}
