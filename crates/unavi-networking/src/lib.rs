use bevy::prelude::*;
use bevy_vrm::VrmBundle;
use thread::{NetworkingThread, NewSession, SessionRequest, SessionResponse};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use unavi_avatar::{default_character_animations, AvatarBundle, DEFAULT_VRM};
use unavi_constants::player::PLAYER_HEIGHT;
use unavi_player::LocalPlayer;
use unavi_world::{InstanceRecord, InstanceServer};
use wired_world::datagram_capnp;

mod thread;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<thread::NetworkingThread>().add_systems(
            FixedUpdate,
            (
                connect_to_instances,
                handle_session_response,
                publish_transform,
            ),
        );
    }
}

#[derive(Component)]
struct Session {
    pub sender: UnboundedSender<SessionRequest>,
    pub receiver: UnboundedReceiver<SessionResponse>,
}

fn connect_to_instances(
    mut commands: Commands,
    runtime: Res<NetworkingThread>,
    to_open: Query<(Entity, &InstanceServer, &InstanceRecord), Without<Session>>,
) {
    for (entity, server, record) in to_open.iter() {
        let address = server.0.clone();
        let record_id = record.0.record_id.clone();

        let (send_req, recv_req) = tokio::sync::mpsc::unbounded_channel::<SessionRequest>();
        let (send_res, recv_res) = tokio::sync::mpsc::unbounded_channel::<SessionResponse>();

        if let Err(e) = runtime.sender.send(NewSession {
            address,
            receiver: recv_req,
            record_id,
            sender: send_res,
        }) {
            error!("{}", e);
            continue;
        }

        commands.entity(entity).insert((
            Session {
                receiver: recv_res,
                sender: send_req,
            },
            LastTransformPublish(0.0),
        ));
    }
}

/// Tickrate of the server, in seconds.
/// We don't want to publish data faster than this rate.
#[derive(Component)]
struct Tickrate(f32);

#[derive(Component)]
struct PlayerId(u16);

fn handle_session_response(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut players: Query<(&PlayerId, &mut Transform)>,
    mut sessions: Query<(Entity, &mut Session)>,
    mut spawned: Local<Vec<u16>>,
) {
    for (entity, mut session) in sessions.iter_mut() {
        if let Ok(res) = session.receiver.try_recv() {
            match res {
                SessionResponse::Tickrate(tickrate) => {
                    commands.entity(entity).insert(Tickrate(tickrate));
                }
                SessionResponse::PlayerTransform {
                    player,
                    rotation,
                    translation,
                } => {
                    if spawned.contains(&player) {
                        for (player_id, mut transform) in players.iter_mut() {
                            if player_id.0 != player {
                                continue;
                            }

                            transform.rotation.x = rotation[0];
                            transform.rotation.y = rotation[1];
                            transform.rotation.z = rotation[2];
                            transform.rotation.w = rotation[3];

                            transform.translation.x = translation[0];
                            transform.translation.y = translation[1];
                            transform.translation.z = translation[2];
                        }
                    } else {
                        info!("Spawning player {}", player);
                        spawned.push(player);

                        commands.spawn((
                            AvatarBundle::new(default_character_animations(&asset_server)),
                            PlayerId(player),
                            VrmBundle {
                                vrm: asset_server.load(DEFAULT_VRM),
                                scene_bundle: SceneBundle {
                                    transform: Transform {
                                        translation: Vec3::from_array(translation),
                                        rotation: Quat::from_array(rotation),
                                        ..default()
                                    },
                                    ..default()
                                },
                                ..default()
                            },
                        ));
                    }
                }
            };
        }
    }
}

#[derive(Component)]
struct LastTransformPublish(f32);

fn publish_transform(
    mut sessions: Query<(&Session, &Tickrate, &mut LastTransformPublish)>,
    players: Query<&Transform, With<LocalPlayer>>,
    time: Res<Time>,
) {
    let elapsed = time.elapsed_seconds();

    if let Some(transform) = players.iter().next() {
        for (session, interval, mut last) in sessions.iter_mut() {
            let delta = elapsed - last.0;

            if delta > interval.0 {
                last.0 = elapsed;
            }

            let mut msg = capnp::message::Builder::new_default();
            let mut root = msg.init_root::<datagram_capnp::publish_transform::Builder>();

            let mut translation = root.reborrow().init_translation();
            translation.set_x(transform.translation.x);
            translation.set_y(transform.translation.y - (PLAYER_HEIGHT / 2.0));
            translation.set_z(transform.translation.z);

            let mut rotation = root.init_rotation();
            rotation.set_x(transform.rotation.x);
            rotation.set_y(transform.rotation.y);
            rotation.set_z(transform.rotation.z);
            rotation.set_w(transform.rotation.w);

            if let Err(e) = session.sender.send(SessionRequest::SendDatagram(msg)) {
                error!("Failed to send: {}", e);
            }
        }
    }
}
