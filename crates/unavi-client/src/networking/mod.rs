use bevy::prelude::*;
use tokio::io::AsyncReadExt;
use unavi_server_service::{ControlServiceClient, from_server::StreamHeader};
use wtransport::{Connection, RecvStream};

mod transform;
mod voice;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, _app: &mut App) {}
}

pub struct SpaceConnection {
    pub connection: Connection,
    pub _control: ControlServiceClient,
}

pub async fn handle_space_connection(space: SpaceConnection) -> anyhow::Result<()> {
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
