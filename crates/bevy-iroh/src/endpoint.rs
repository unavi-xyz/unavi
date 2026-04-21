use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use bevy::prelude::*;
use iroh::{Endpoint, endpoint::presets::N0};
use tracing::{error, info};

use crate::{IrohEndpoint, LoadEndpoint};

pub fn on_load_endpoint(trigger: On<LoadEndpoint>, mut commands: Commands) {
    let (tx, rx) = std::sync::mpsc::channel();
    let opts = trigger.event().clone();

    unavi_wasm_compat::spawn_thread(async move {
        let mut delay_secs = 4;

        loop {
            match init_endpoint(&opts).await {
                Ok(val) => {
                    tx.send(val).expect("send endpoint");
                    break;
                }
                Err(err) => {
                    error!(?err, "failed to init endpoint");
                    n0_future::time::sleep(Duration::from_secs(delay_secs)).await;
                    delay_secs *= 2;
                }
            }
        }
    });

    commands.spawn(LoadingEndpoint(Arc::new(Mutex::new(rx))));
}

#[derive(Component)]
pub struct LoadingEndpoint(Arc<Mutex<std::sync::mpsc::Receiver<Endpoint>>>);

pub fn recieve_endpoint(mut commands: Commands, loading: Query<(Entity, &LoadingEndpoint)>) {
    let Some((ent, rx)) = loading.iter().next() else {
        return;
    };

    let Ok(lock) = rx.0.try_lock() else {
        return;
    };

    let Ok(endpoint) = lock.try_recv() else {
        return;
    };

    commands
        .entity(ent)
        .insert(IrohEndpoint(endpoint))
        .remove::<LoadingEndpoint>();
}

async fn init_endpoint(opts: &LoadEndpoint) -> anyhow::Result<Endpoint> {
    let mut endpoint = Endpoint::builder(N0);

    if opts.discovery_mdns {
        endpoint =
            endpoint.address_lookup(iroh::address_lookup::mdns::MdnsAddressLookup::builder());
    }

    let endpoint = endpoint.bind().await?;
    info!("spawned endpoint: {}", endpoint.id());

    Ok(endpoint)
}
