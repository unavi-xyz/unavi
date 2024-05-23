use bevy::prelude::*;
use session_runner::{InstanceAction, NewSession, SessionRunner};
use tokio::sync::mpsc::UnboundedSender;
use unavi_world::{InstanceRecord, InstanceServer};

pub mod session_runner;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<session_runner::SessionRunner>()
            .add_systems(Update, (connect_to_instances, publish_transform));
    }
}

#[derive(Component)]
struct InstanceSession {
    pub sender: UnboundedSender<InstanceAction>,
}

fn connect_to_instances(
    mut commands: Commands,
    sessions: Res<SessionRunner>,
    to_open: Query<(Entity, &InstanceServer, &InstanceRecord), Without<InstanceSession>>,
) {
    for (entity, server, record) in to_open.iter() {
        let address = server.0.clone();
        let record_id = record.0.record_id.clone();

        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<InstanceAction>();

        if let Err(e) = sessions.sender.send(NewSession {
            address,
            receiver,
            record_id,
        }) {
            error!("{}", e);
            continue;
        }

        commands
            .entity(entity)
            .insert((InstanceSession { sender }, LastTransformPublish(0.0)));
    }
}

/// How often to publish the player transform, in seconds.
/// Determined by the server.
#[derive(Component, Deref, DerefMut)]
struct InstancePublishInterval(f32);

#[derive(Component, Deref, DerefMut)]
struct LastTransformPublish(f32);

fn publish_transform(
    time: Res<Time>,
    mut sessions: Query<(
        &InstanceSession,
        &InstancePublishInterval,
        &mut LastTransformPublish,
    )>,
) {
    let elapsed = time.elapsed_seconds();

    for (session, interval, mut last) in sessions.iter_mut() {
        let delta = elapsed - last.0;

        if delta > interval.0 {
            **last = elapsed;
        }

        if let Err(e) = session
            .sender
            .send(InstanceAction::SendDatagram(Box::new([])))
        {
            error!("Failed to publish transform: {}", e);
        }
    }
}
