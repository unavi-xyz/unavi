use std::sync::mpsc::SyncSender;

use bevy::prelude::*;
use tokio::io::AsyncReadExt;
use unavi_server_service::from_server::{ControlMessage, StreamHeader};
use wtransport::RecvStream;

use crate::space::streams::transform::TransformChannels;

pub mod control;
pub mod publish;
pub mod transform;
mod voice;

pub const PFRAME_ROTATION_SCALE: f32 = i16::MAX as f32;
pub const PFRAME_TRANSLATION_SCALE: f32 = 1000.0;

pub const JOINT_ROTATION_EPSILON: f32 = 1.0 / PFRAME_ROTATION_SCALE;

pub async fn recv_stream(
    mut stream: RecvStream,
    transform_channels: TransformChannels,
    control_tx: SyncSender<ControlMessage>,
) -> anyhow::Result<()> {
    let header_len = stream.read_u16_le().await? as usize;
    let mut header_buf = vec![0; header_len];
    stream.read_exact(&mut header_buf).await?;

    let (header, _) = bincode::decode_from_slice(&header_buf, bincode::config::standard())?;

    match header {
        StreamHeader::Transform => {
            transform::recv_transform_stream(stream, transform_channels).await?;
        }
        StreamHeader::Voice => {
            voice::recv_voice_stream(stream).await?;
        }
        StreamHeader::Control => {
            control::recv_control_stream(stream, control_tx).await?;
        }
    }

    Ok(())
}
