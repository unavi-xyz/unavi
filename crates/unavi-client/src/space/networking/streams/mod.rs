use unavi_server_service::from_server::{ControlMessage, StreamHeader};
use wtransport::RecvStream;

pub mod control;
pub mod publish;
pub mod transform;
mod voice;

pub use transform::TransformChannels;

pub const PFRAME_ROTATION_SCALE: f32 = i16::MAX as f32;
pub const PFRAME_TRANSLATION_SCALE: f32 = 1000.0;

pub const JOINT_ROTATION_EPSILON: f32 = 1.0 / PFRAME_ROTATION_SCALE;

pub async fn recv_stream(
    mut stream: RecvStream,
    transform_channels: TransformChannels,
    control_tx: flume::Sender<ControlMessage>,
    #[cfg(feature = "devtools-network")] connect_url: String,
) -> anyhow::Result<()> {
    let mut header_buf = [0u8; 1];
    stream.read_exact(&mut header_buf).await?;

    let (header, _) = bincode::decode_from_slice(&header_buf, bincode::config::standard())?;

    match header {
        StreamHeader::TransformIFrame => {
            transform::recv_iframe_stream(
                stream,
                transform_channels,
                #[cfg(feature = "devtools-network")]
                connect_url,
            )
            .await?;
        }
        StreamHeader::TransformPFrame => {
            transform::recv_pframe_stream(
                stream,
                transform_channels,
                #[cfg(feature = "devtools-network")]
                connect_url,
            )
            .await?;
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
