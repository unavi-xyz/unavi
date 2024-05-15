use anyhow::{anyhow, Result};
use bevy::log::info;
use capnp_rpc::{rpc_twoparty_capnp::Side, twoparty::VatNetwork, RpcSystem};
use xwt_core::{base::Session, session::stream::OpeningBi};
use xwt_futures_io::{read::ReadCompat, write::WriteCompat};

pub async fn handle_session<T: Session + 'static>(session: T) -> Result<()> {
    info!("Opening bi stream.");
    let opening = session.open_bi().await.map_err(|e| anyhow!("{}", e))?;
    let (send, recv) = opening.wait_bi().await.map_err(|e| anyhow!("{}", e))?;
    info!("Stream opened.");

    let reader = ReadCompat::<T>::new(recv);
    let writer = WriteCompat::<T>::new(send);

    let rpc_network = VatNetwork::new(
        Box::pin(reader),
        Box::pin(writer),
        Side::Client,
        Default::default(),
    );
    let mut rpc_system = RpcSystem::new(Box::new(rpc_network), None);

    let world_server: wired_world::world_server_capnp::world_server::Client =
        rpc_system.bootstrap(Side::Server);

    tokio::task::spawn_local(rpc_system);

    tokio::task::spawn_local(async move {
        let mut request = world_server.join_instance_request();
        request.get().init_instance().set_record_id("test");

        let _reply = request.send().promise.await;
    });

    Ok(())
}
