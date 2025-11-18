use bevy::log::*;
use tokio::task::AbortHandle;

use crate::space::streams::recv_stream;

use super::host::HostConnection;

/// Spawns a task to accept incoming streams from the server.
/// Returns an AbortHandle for cleanup.
pub fn spawn_stream_accept_task(
    host: HostConnection,
    #[cfg(feature = "devtools-network")] connect_url: String,
) -> AbortHandle {
    let connection = host.connection.clone();
    let transform_channels = host.transform_channels.clone();
    let control_tx = host.control_tx.clone();

    let task = tokio::spawn(async move {
        while let Ok(stream) = connection.accept_uni().await {
            let transform_channels = transform_channels.clone();
            let control_tx = control_tx.clone();

            #[cfg(feature = "devtools-network")]
            let connect_url = connect_url.clone();

            tokio::spawn(async move {
                if let Err(e) = recv_stream(
                    stream,
                    transform_channels,
                    control_tx,
                    #[cfg(feature = "devtools-network")]
                    connect_url,
                )
                .await
                {
                    error!("Error handling stream: {e:?}");
                };
            });
        }
    });

    task.abort_handle()
}
