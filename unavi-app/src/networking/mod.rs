use aeronet::{AsyncRuntime, ClientTransportPlugin, TryFromBytes, TryIntoBytes};
use aeronet_wt_native::{Channels, OnChannel, WebTransportClient};
use anyhow::Result;
use bevy::prelude::*;
use std::time::Duration;
use wtransport::ClientConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Channels)]
#[channel_kind(Datagram)]
struct AppChannel;

#[derive(Debug, Clone, PartialEq, Eq, Hash, OnChannel)]
#[channel_type(AppChannel)]
#[on_channel(AppChannel)]
struct AppMessage(String);

impl TryFromBytes for AppMessage {
    fn try_from_bytes(buf: &[u8]) -> Result<Self> {
        String::from_utf8(buf.to_vec())
            .map(AppMessage)
            .map_err(Into::into)
    }
}

impl TryIntoBytes for AppMessage {
    fn try_into_bytes(self) -> Result<Vec<u8>> {
        Ok(self.0.into_bytes())
    }
}

type Client = WebTransportClient<AppMessage, AppMessage, AppChannel>;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ClientTransportPlugin::<_, _, Client>::default())
            .init_resource::<AsyncRuntime>()
            .add_event::<JoinWorld>()
            .add_systems(Startup, setup)
            .add_systems(Update, join_world);
    }
}

fn setup(mut commands: Commands, rt: Res<AsyncRuntime>) {
    match create(&rt) {
        Ok(client) => {
            commands.insert_resource(client);
        }
        Err(err) => panic!("Failed to create server: {err:#}"),
    }
}

fn create(rt: &AsyncRuntime) -> Result<Client> {
    let config = ClientConfig::builder()
        .with_bind_default()
        .with_no_cert_validation()
        .keep_alive_interval(Some(Duration::from_secs(5)))
        .build();

    let (front, back) = aeronet_wt_native::create_client(config);

    rt.0.spawn(async move {
        back.start().await.unwrap();
    });

    Ok(front)
}

#[derive(Event)]
pub struct JoinWorld {
    pub world_id: u32,
}

fn join_world(mut events: EventReader<JoinWorld>, client: Res<Client>) {
    for event in events.read() {
        info!("Joining world {}", event.world_id);

        let url = "https://localhost:4433";
        client.connect(url);
    }
}
