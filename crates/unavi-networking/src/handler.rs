use anyhow::{anyhow, Result};
use bevy::log::info;
use capnp_rpc::{rpc_twoparty_capnp::Side, twoparty::VatNetwork, RpcSystem};
use wired_world::world_server_capnp::world_server::Client;
use xwt_core::{base::Session, session::stream::OpeningBi};
use xwt_futures_io::{read::ReadCompat, write::WriteCompat};

pub async fn instance_session(address: String, record_id: String) -> Result<()> {
    let session = start_session(&address).await?;
    let world_server = create_world_server_rpc(session).await?;

    join_instance(&world_server, record_id).await?;

    Ok(())
}

async fn create_world_server_rpc<T: Session + 'static>(session: T) -> Result<Client> {
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

    let world_server = rpc_system.bootstrap(Side::Server);

    tokio::task::spawn_local(rpc_system);

    Ok(world_server)
}

async fn join_instance(world_server: &Client, record_id: String) -> Result<()> {
    let mut request = world_server.join_instance_request();
    request.get().init_instance().set_record_id(record_id);

    let reply = request.send().promise.await?;
    let _response = reply.get()?.get_response()?;

    Ok(())
}

const LOCALHOST: &str = "https://localhost:";

async fn start_session(addr: &str) -> Result<impl Session> {
    let addr = if addr.starts_with(LOCALHOST) {
        addr.replace(LOCALHOST, "https://127.0.0.1:")
    } else {
        addr.to_string()
    };

    info!("Beginning connection process.");
    let session = crate::connect::connect(&addr)
        .await
        .map_err(|e| anyhow!("{}", e))?;

    Ok(session)
}
