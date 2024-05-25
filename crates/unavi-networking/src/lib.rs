use bevy::prelude::*;
use thread::{NetworkingThread, NewSession, SessionRequest, SessionResponse};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use unavi_world::{InstanceRecord, InstanceServer};

mod thread;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<thread::NetworkingThread>().add_systems(
            Update,
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
#[derive(Component, Deref, DerefMut)]
struct Tickrate(f32);

fn handle_session_response(mut commands: Commands, mut sessions: Query<(Entity, &mut Session)>) {
    for (entity, mut session) in sessions.iter_mut() {
        if let Ok(res) = session.receiver.try_recv() {
            match res {
                SessionResponse::Tickrate(tickrate) => {
                    commands.entity(entity).insert(Tickrate(tickrate))
                }
            };
        }
    }
}

#[derive(Component, Deref, DerefMut)]
struct LastTransformPublish(f32);

fn publish_transform(
    time: Res<Time>,
    mut sessions: Query<(&Session, &Tickrate, &mut LastTransformPublish)>,
) {
    let elapsed = time.elapsed_seconds();

    for (session, interval, mut last) in sessions.iter_mut() {
        let delta = elapsed - last.0;

        if delta > interval.0 {
            **last = elapsed;
        }

        if let Err(e) = session
            .sender
            .send(SessionRequest::SendDatagram(Box::new([])))
        {
            error!("Failed to send: {}", e);
        }
    }
}
