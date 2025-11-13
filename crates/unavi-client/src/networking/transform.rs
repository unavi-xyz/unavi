use bevy::prelude::*;
use tokio::io::AsyncReadExt;
use unavi_server_service::from_server::TransformMeta;
use wtransport::RecvStream;

pub async fn handle_transform_stream(mut stream: RecvStream) -> anyhow::Result<()> {
    let meta_len = stream.read_u16().await? as usize;

    let mut meta_buf = vec![0; meta_len];
    stream.read_exact(&mut meta_buf).await?;

    let (meta, _) = bincode::serde::decode_from_slice::<TransformMeta, _>(
        &meta_buf,
        bincode::config::standard(),
    )?;

    info!("Got transform stream: {meta:?}");

    Ok(())
}
