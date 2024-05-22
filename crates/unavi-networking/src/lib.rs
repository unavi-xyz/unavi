use bevy::{prelude::*, utils::tracing::Instrument};
use tokio::{sync::mpsc::UnboundedSender, task::LocalSet};
use unavi_world::{InstanceRecord, InstanceServer};

mod connect;
pub mod handler;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Sessions>()
            .add_systems(Update, connect_to_instances);
    }
}

#[derive(Component)]
pub struct InstanceSession;

#[derive(Resource)]
pub struct Sessions {
    pub sender: UnboundedSender<NewSession>,
}

impl Default for Sessions {
    fn default() -> Self {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<NewSession>();

        let task = async move {
            while let Some(new_session) = receiver.recv().await {
                let span = info_span!("Session", address = new_session.address);

                tokio::task::spawn_local(
                    async move {
                        match handler::handle_instance_session(
                            &new_session.address,
                            new_session.record_id,
                        )
                        .await
                        {
                            Ok(_) => info!("Graceful exit."),
                            Err(e) => error!("{}", e),
                        };
                    }
                    .instrument(span),
                );
            }
        };

        #[cfg(not(target_family = "wasm"))]
        std::thread::spawn(move || {
            let local = LocalSet::new();
            local.spawn_local(task);
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(local);
        });

        #[cfg(target_family = "wasm")]
        let _ = wasm_bindgen_futures::future_to_promise(async move {
            let local = LocalSet::new();
            local.run_until(task).await;
            Ok(wasm_bindgen::JsValue::UNDEFINED)
        });

        Self { sender }
    }
}

pub struct NewSession {
    pub address: String,
    pub record_id: String,
}

pub fn connect_to_instances(
    mut commands: Commands,
    sessions: Res<Sessions>,
    to_open: Query<(Entity, &InstanceServer, &InstanceRecord), Without<InstanceSession>>,
) {
    for (entity, server, record) in to_open.iter() {
        let address = server.0.clone();
        let record_id = record.0.record_id.clone();

        if let Err(e) = sessions.sender.send(NewSession { address, record_id }) {
            error!("{}", e);
            continue;
        }

        commands.entity(entity).insert(InstanceSession);
    }
}
