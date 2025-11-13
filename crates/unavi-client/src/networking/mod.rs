use std::time::Duration;

use bevy::prelude::*;
use tokio::io::AsyncReadExt;
use unavi_server_service::{ControlServiceClient, from_server::StreamHeader};
use wtransport::{Connection, RecvStream};
use xdid::core::did_url::DidUrl;

use crate::networking::tickrate::{SetTickrate, TICKRATE_QUEUE};

mod publish;
mod tickrate;
mod transform;
mod voice;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                publish::publish_transform_data,
                tickrate::set_space_tickrates,
            ),
        );
    }
}

pub struct SpaceSession {
    pub connection: Connection,
    pub control: ControlServiceClient,
}

const MAX_TICKRATE: u64 = 1_000;
const MIN_TICKRATE: u64 = 25;

pub async fn handle_space_session(
    space: SpaceSession,
    space_id: String,
    space_url: DidUrl,
) -> anyhow::Result<()> {
    let tickrate_ms = space
        .control
        .tickrate_ms(tarpc::context::current())
        .await?
        .clamp(MIN_TICKRATE, MAX_TICKRATE);

    TICKRATE_QUEUE.0.send(SetTickrate {
        space_url,
        tickrate: Duration::from_millis(tickrate_ms),
    })?;

    space
        .control
        .join_space(tarpc::context::current(), space_id.clone())
        .await?
        .map_err(|e| anyhow::anyhow!("rpc error: {e}"))?;
    info!("Joined space {space_id}");

    loop {
        let stream = space.connection.accept_uni().await?;

        tokio::spawn(async move {
            if let Err(e) = handle_stream(stream).await {
                error!("Error handling stream: {e:?}");
            };
        });
    }
}

async fn handle_stream(mut stream: RecvStream) -> anyhow::Result<()> {
    let header_len = stream.read_u16().await? as usize;

    let mut header_buf = vec![0; header_len];
    stream.read_exact(&mut header_buf).await?;

    let (header, _) = bincode::decode_from_slice(&header_buf, bincode::config::standard())?;

    match header {
        StreamHeader::Transform => {
            transform::handle_transform_stream(stream).await?;
        }
        StreamHeader::Voice => {
            voice::handle_voice_stream(stream).await?;
        }
    }

    Ok(())
}
