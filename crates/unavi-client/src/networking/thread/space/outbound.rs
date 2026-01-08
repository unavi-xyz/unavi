use iroh::EndpointId;

use crate::networking::thread::NetworkThreadState;

pub async fn handle_outbound(state: NetworkThreadState, player: EndpointId) -> anyhow::Result<()> {
    let connection = state.endpoint.connect(player, super::ALPN).await?;

    let (_tx, _rx) = connection.accept_bi().await?;

    Ok(())
}
