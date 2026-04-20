use std::time::Duration;

use bevy::prelude::*;
use iroh::{Endpoint, endpoint::presets::N0};
use tracing::{error, info};

#[derive(Component)]
pub struct IrohEndpoint(pub Endpoint);

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, States)]
pub enum NetThreadLoadState {
    #[default]
    Loading,
    Ready,
}

pub fn spawn_net_thread(
    mut commands: Commands,
    mut state: ResMut<NextState<NetThreadLoadState>>,
    mut rx_state: Local<Option<std::sync::mpsc::Receiver<Endpoint>>>,
) {
    if rx_state.is_none() {
        let (tx, rx) = std::sync::mpsc::channel();

        unavi_wasm_compat::spawn_thread(async move {
            let mut delay_secs = 4;

            loop {
                match init_endpoint().await {
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

        *rx_state = Some(rx);
    }

    if let Some(rx) = rx_state.as_ref() {
        match rx.try_recv() {
            Ok(endpoint) => {
                commands.spawn(IrohEndpoint(endpoint));
                state.set(NetThreadLoadState::Ready);
            }
            Err(err) => {
                error!(?err, "failed to recieve endpoint");
                *rx_state = None;
            }
        }
    }
}

async fn init_endpoint() -> anyhow::Result<Endpoint> {
    let endpoint = Endpoint::builder(N0);

    #[cfg(feature = "mdns")]
    let endpoint =
        endpoint.address_lookup(iroh::address_lookup::mdns::MdnsAddressLookup::builder());

    let endpoint = endpoint.bind().await?;
    info!("local endpoint: {}", endpoint.id());

    Ok(endpoint)
}
