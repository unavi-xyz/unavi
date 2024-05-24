use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use tokio::sync::mpsc::{error::SendError, UnboundedReceiver, UnboundedSender};
use tracing::debug;

#[derive(Debug)]
pub struct SessionMessage {
    pub command: SessionCommand,
    pub player_id: usize,
}

#[derive(Debug)]
pub enum SessionCommand {
    Disconnect,
    JoinInstance { id: String },
    LeaveInstance { id: String },
    SetTransform(Transform),
}

#[derive(Debug, Default)]
pub struct Transform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
}

pub async fn process_commands(
    sender: UnboundedSender<SessionMessage>,
    mut receiver: UnboundedReceiver<SessionMessage>,
) -> Result<(), SendError<SessionMessage>> {
    let mut instances = HashMap::<String, Instance>::default();
    let mut players = HashMap::<usize, Player>::default();

    while let Some(msg) = receiver.recv().await {
        debug!("Processing command msg: {:?}", msg);

        match msg.command {
            SessionCommand::Disconnect => {
                for (id, instance) in instances.iter_mut() {
                    if instance.players.contains(&msg.player_id) {
                        sender.send(SessionMessage {
                            command: SessionCommand::LeaveInstance { id: id.to_owned() },
                            player_id: msg.player_id,
                        })?;
                    }
                }

                players.remove(&msg.player_id);
            }
            SessionCommand::JoinInstance { id } => {
                let instance = match instances.get_mut(&id) {
                    Some(i) => i,
                    None => {
                        instances.insert(id.clone(), Instance::default());
                        instances.get_mut(&id).unwrap()
                    }
                };

                instance.players.insert(msg.player_id);
            }
            SessionCommand::LeaveInstance { id } => {
                let instance = match instances.get_mut(&id) {
                    Some(i) => i,
                    None => continue,
                };

                instance.players.remove(&msg.player_id);

                if instance.players.is_empty() {
                    instances.remove(&id);
                }
            }
            SessionCommand::SetTransform(transform) => {
                let player = match players.get_mut(&msg.player_id) {
                    Some(i) => i,
                    None => {
                        players.insert(msg.player_id, Player::default());
                        players.get_mut(&msg.player_id).unwrap()
                    }
                };

                player.transform = transform;
            }
        }
    }

    Ok(())
}

#[derive(Default)]
struct Instance {
    players: HashSet<usize>,
}

#[derive(Default)]
struct Player {
    transform: Transform,
}
