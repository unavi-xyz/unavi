use std::{
    collections::{btree_map::Keys, BTreeMap, HashMap, HashSet},
    time::Duration,
};

use thiserror::Error;
use tokio::sync::mpsc::{error::SendError, UnboundedReceiver, UnboundedSender};
use tracing::debug;

pub const TICKRATE: f32 = 1.0 / 60.0;

#[derive(Debug)]
pub struct IncomingEvent {
    pub command: IncomingCommand,
    pub player_id: usize,
}

#[derive(Debug)]
pub enum IncomingCommand {
    Disconnect,
    JoinWorld {
        id: String,
    },
    LeaveWorld {
        id: String,
    },
    NewPlayer {
        sender: UnboundedSender<OutgoingEvent>,
    },
    SetTransform(Transform),
}

#[derive(Clone, Debug, Default)]
pub struct Transform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
}

#[derive(Debug)]
pub enum OutgoingEvent {
    PlayerJoined { id: usize },
    PlayerLeft { id: usize },
    Transforms(Vec<Transform>),
}

#[derive(Error, Debug)]
pub enum UpdateLoopError {
    #[error(transparent)]
    SendIncoming(#[from] SendError<IncomingEvent>),
    #[error(transparent)]
    SendOutgoing(#[from] SendError<OutgoingEvent>),
}

pub async fn update_loop(
    sender: UnboundedSender<IncomingEvent>,
    mut receiver: UnboundedReceiver<IncomingEvent>,
) -> Result<(), UpdateLoopError> {
    let duration = Duration::from_secs_f32(TICKRATE);
    let mut worlds = HashMap::<String, World>::default();
    let mut players = HashMap::<usize, Player>::default();

    loop {
        tokio::time::sleep(duration).await;

        while let Ok(msg) = receiver.try_recv() {
            debug!("Processing command msg: {:?}", msg);

            match msg.command {
                IncomingCommand::Disconnect => {
                    for (id, world) in worlds.iter_mut() {
                        if world.players.contains(&msg.player_id) {
                            sender.send(IncomingEvent {
                                command: IncomingCommand::LeaveWorld { id: id.to_owned() },
                                player_id: msg.player_id,
                            })?;
                        }
                    }

                    players.remove(&msg.player_id);
                }
                IncomingCommand::JoinWorld { id } => {
                    let world = match worlds.get_mut(&id) {
                        Some(i) => i,
                        None => {
                            worlds.insert(id.clone(), World::default());
                            worlds.get_mut(&id).unwrap()
                        }
                    };

                    let player = players.get_mut(&msg.player_id).unwrap();

                    for other_id in world.players.iter() {
                        player.known_players.add(*other_id);
                        player
                            .sender
                            .send(OutgoingEvent::PlayerJoined { id: *other_id })?;
                    }

                    for other_id in world.players.iter() {
                        let other = players.get_mut(other_id).unwrap();
                        other.known_players.add(msg.player_id);
                        other
                            .sender
                            .send(OutgoingEvent::PlayerJoined { id: msg.player_id })?;
                    }

                    world.players.insert(msg.player_id);
                }
                IncomingCommand::LeaveWorld { id } => {
                    let world = match worlds.get_mut(&id) {
                        Some(i) => i,
                        None => continue,
                    };

                    world.players.remove(&msg.player_id);

                    for player_id in world.players.iter() {
                        let player = players.get_mut(player_id).unwrap();
                        player.known_players.remove(*player_id);
                        player
                            .sender
                            .send(OutgoingEvent::PlayerLeft { id: msg.player_id })?;
                    }

                    if world.players.is_empty() {
                        worlds.remove(&id);
                    }
                }
                IncomingCommand::NewPlayer { sender } => {
                    players.insert(
                        msg.player_id,
                        Player {
                            known_players: Default::default(),
                            sender,
                            transform: Default::default(),
                        },
                    );
                }
                IncomingCommand::SetTransform(transform) => {
                    let player = players.get_mut(&msg.player_id).unwrap();
                    player.transform = transform;
                }
            }
        }

        for player in players.values() {
            let mut transforms = Vec::new();

            for player_id in player.known_players.iter() {
                let other = players.get(player_id).unwrap();
                transforms.push(other.transform.clone());
            }

            player.sender.send(OutgoingEvent::Transforms(transforms))?;
        }
    }
}

#[derive(Default)]
struct World {
    players: HashSet<usize>,
}

struct Player {
    known_players: KnownPlayers,
    sender: UnboundedSender<OutgoingEvent>,
    transform: Transform,
}

#[derive(Default)]
struct KnownPlayers {
    /// Maps player id -> reference count.
    map: BTreeMap<usize, usize>,
}

impl KnownPlayers {
    fn add(&mut self, id: usize) {
        if let Some(count) = self.map.get(&id) {
            self.map.insert(id, count + 1);
        } else {
            self.map.insert(id, 1);
        }
    }

    fn remove(&mut self, id: usize) {
        if let Some(count) = self.map.get(&id) {
            if *count >= 2 {
                self.map.insert(id, count - 1);
            } else {
                self.map.remove(&id);
            }
        }
    }

    fn iter(&self) -> Keys<usize, usize> {
        self.map.keys()
    }
}
