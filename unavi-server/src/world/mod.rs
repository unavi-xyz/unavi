use aeronet::{
    AsyncRuntime, DisconnectClient, FromClient, RemoteClientConnected, RemoteClientDisconnected,
    ServerTransport, ServerTransportPlugin, ToClient, TryFromBytes, TryIntoBytes,
};
use aeronet_wt_native::{Channels, OnChannel, WebTransportServer};
use anyhow::Result;
use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    log::LogPlugin,
    prelude::*,
};
use std::net::SocketAddr;
use std::time::Duration;
use wtransport::{tls::Certificate, ServerConfig};

pub mod cert;

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

type Server = WebTransportServer<AppMessage, AppMessage, AppChannel>;

pub struct WorldOptions {
    pub address: SocketAddr,
    pub cert_pair: CertPair,
}

#[derive(Resource)]
pub struct CertPair {
    pub cert: Vec<u8>,
    pub key: Vec<u8>,
}

pub async fn start_server(opts: WorldOptions) -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins((
            LogPlugin {
                level: tracing::Level::DEBUG,
                ..default()
            },
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(100))),
            ServerTransportPlugin::<_, _, Server>::default(),
        ))
        .init_resource::<AsyncRuntime>()
        .insert_resource(opts.cert_pair)
        .add_systems(Startup, setup)
        .add_systems(Update, (reply, log))
        .run();

    Ok(())
}

fn setup(mut commands: Commands, rt: Res<AsyncRuntime>, cert: Res<CertPair>) {
    let cert = Certificate::new(vec![cert.cert.clone()], cert.key.clone());

    match create(&rt, cert) {
        Ok(server) => {
            commands.insert_resource(server);
        }
        Err(err) => panic!("Failed to create server: {err:#}"),
    }
}

fn create(rt: &AsyncRuntime, cert: Certificate) -> Result<Server> {
    let port = 4433;

    let config = ServerConfig::builder()
        .with_bind_default(port)
        .with_certificate(cert)
        .keep_alive_interval(Some(Duration::from_secs(5)))
        .build();

    let (front, back) = aeronet_wt_native::create_server(config);

    rt.0.spawn(async move {
        back.start().await.unwrap();
    });

    info!("World listening on 127.0.0.1:{}", port);

    Ok(front)
}

fn log(
    server: Res<Server>,
    mut connected: EventReader<RemoteClientConnected>,
    mut disconnected: EventReader<RemoteClientDisconnected>,
) {
    for RemoteClientConnected(client) in connected.read() {
        info!("Client {client} connected");
        info!("  Info: {:?}", server.client_info(*client));
    }

    for RemoteClientDisconnected(client, reason) in disconnected.read() {
        info!(
            "Client {client} disconnected: {:#}",
            aeronet::error::as_pretty(reason),
        );
        info!("  Info: {:?}", server.client_info(*client));
    }
}

fn reply(
    mut recv: EventReader<FromClient<AppMessage>>,
    mut send: EventWriter<ToClient<AppMessage>>,
    mut disconnect: EventWriter<DisconnectClient>,
    mut exit: EventWriter<AppExit>,
) {
    for FromClient(client, msg) in recv.read() {
        info!("From {client}: {:?}", msg.0);
        match msg.0.as_str() {
            "dc" => disconnect.send(DisconnectClient(*client)),
            "stop" => exit.send(AppExit),
            msg => {
                let msg = format!("You sent: {}", msg);
                send.send(ToClient(*client, AppMessage(msg)));
            }
        }
    }
}
