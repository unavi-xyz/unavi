use anyhow::{anyhow, Result};
use bevy::log::info;
use xwt_core::{base::Session, session::stream::OpeningBi, stream::Write};

pub async fn handle_session(session: impl Session) -> Result<()> {
    info!("Opening bi stream.");
    let opening = session.open_bi().await.map_err(|e| anyhow!("{}", e))?;
    let (mut send, _recv) = opening.wait_bi().await.map_err(|e| anyhow!("{}", e))?;

    info!("Stream opened. Sending data.");
    let data = "Hello from app!".to_string();
    send.write(data.as_bytes())
        .await
        .map_err(|e| anyhow!("{}", e))?;

    Ok(())
}
