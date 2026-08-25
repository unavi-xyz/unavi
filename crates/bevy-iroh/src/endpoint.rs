use std::time::Duration;

use bevy::prelude::*;
use iroh::{
    Endpoint,
    SecretKey,
    endpoint::presets::N0,
    endpoint_info::AddrFilter,
};
use tracing::{
    error,
    info,
};
use unavi_util::async_task::spawn_async_task;

use crate::router::RouterBuilderFns;

#[derive(Component)]
#[require(RouterBuilderFns)]
pub struct IrohEndpoint(pub Endpoint);

#[derive(Event, Clone)]
pub struct LoadEndpoint {
    pub filter:     AddrFilter,
    /// Fixes the endpoint id across restarts, so a peer that recorded this
    /// node's address can still reach it.
    pub secret_key: SecretKey,
}

pub(crate) fn on_load_endpoint(trigger: On<LoadEndpoint>, mut commands: Commands) {
    let (tx, rx) = async_channel::bounded(1);
    let opts = trigger.event().clone();

    spawn_async_task(async move {
        let mut delay_secs = 4;

        loop {
            match init_endpoint(&opts).await {
                Ok(val) => {
                    tx.send(val).await.expect("send endpoint");
                    break;
                }
                Err(err) => {
                    error!(?err, "Failed to init endpoint");
                    n0_future::time::sleep(Duration::from_secs(delay_secs)).await;
                    delay_secs = delay_secs.wrapping_mul(2).min(300);
                }
            }
        }
    });

    commands.spawn(LoadingEndpoint(rx));
}

#[derive(Component)]
pub(crate) struct LoadingEndpoint(async_channel::Receiver<Endpoint>);

pub(crate) fn receive_endpoint(mut commands: Commands, loading: Query<(Entity, &LoadingEndpoint)>) {
    for (ent, rx) in loading.iter() {
        let Ok(endpoint) = rx.0.try_recv() else {
            continue;
        };
        commands
            .entity(ent)
            .insert(IrohEndpoint(endpoint))
            .remove::<LoadingEndpoint>();
    }
}

async fn init_endpoint(opts: &LoadEndpoint) -> anyhow::Result<Endpoint> {
    let endpoint = Endpoint::builder(N0)
        .addr_filter(opts.filter.clone())
        .secret_key(opts.secret_key.clone());

    let endpoint = endpoint.bind().await?;
    info!("Spawned endpoint: {}", endpoint.id());

    Ok(endpoint)
}
