use std::{sync::LazyLock, time::Duration};

use iroh::endpoint::{Connection, RecvStream, SendStream};
use tokio::sync::Mutex;

use crate::connection::shared::{
    StreamIdent,
    types::{IFrame, pose::Pose},
};

static AGENT_POSE_STORE: LazyLock<Mutex<Pose<IFrame>>> =
    LazyLock::new(|| Mutex::new(Pose::default()));

pub async fn send_agent_stream(connection: &Connection) -> anyhow::Result<()> {
    let (mut tx, rx) = connection.open_bi().await?;
    StreamIdent::Agent.write(&mut tx).await?;

    let tickrate = Duration::from_millis(100);

    // Spawn tickrate management task

    loop {
        // Read pose from store

        // Convert to I-Frame or P-Frame

        // Send

        n0_future::time::sleep(tickrate).await;
    }
}

pub async fn recv_agent_stream(tx: SendStream, rx: RecvStream) -> anyhow::Result<()> {
    Ok(())
}
