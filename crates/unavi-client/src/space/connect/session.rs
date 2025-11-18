use std::time::Duration;

use bevy::log::*;
use xdid::core::did_url::DidUrl;

use super::host::HostConnection;
use crate::space::tickrate::{SetTickrate, TICKRATE_QUEUE};

const MIN_TICKRATE: u64 = 25;
const MAX_TICKRATE: u64 = 1_000;

/// Joins a space session by calling the server RPC and setting tickrate.
pub async fn join_space_session(
    host: &HostConnection,
    space_id: String,
    space_url: DidUrl,
) -> anyhow::Result<()> {
    let tickrate_ms = fetch_tickrate(host).await?;
    set_tickrate(space_url, tickrate_ms)?;
    join_space(host, space_id).await?;

    Ok(())
}

async fn fetch_tickrate(host: &HostConnection) -> anyhow::Result<u64> {
    let tickrate_ms = host
        .control
        .tickrate_ms(tarpc::context::current())
        .await?
        .clamp(MIN_TICKRATE, MAX_TICKRATE);

    Ok(tickrate_ms)
}

fn set_tickrate(space_url: DidUrl, tickrate_ms: u64) -> anyhow::Result<()> {
    TICKRATE_QUEUE.0.send(SetTickrate {
        space_url,
        tickrate: Duration::from_millis(tickrate_ms),
    })?;

    Ok(())
}

async fn join_space(host: &HostConnection, space_id: String) -> anyhow::Result<()> {
    host.control
        .join_space(tarpc::context::current(), space_id.clone())
        .await?
        .map_err(|e| anyhow::anyhow!("rpc error: {e}"))?;

    info!("Joined space {space_id}");

    Ok(())
}
