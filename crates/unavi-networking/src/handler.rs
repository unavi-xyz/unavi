use anyhow::{anyhow, Result};
use bevy::{
    log::{debug, error, info, info_span},
    utils::tracing::Instrument,
};
use capnp_rpc::{rpc_twoparty_capnp::Side, twoparty::VatNetwork, RpcSystem};
use wired_world::world_server_capnp::{result::Which, world_server::Client};
use xwt_core::{base::Session, session::stream::OpeningBi};
use xwt_futures_io::{read::ReadCompat, write::WriteCompat};

use crate::{InstanceAction, NewSession};

pub async fn handle_session(
    NewSession {
        address,
        mut action_receiver,
        record_id,
    }: NewSession,
) -> Result<()> {
    let session = crate::connect::connect(&address)
        .await
        .map_err(|e| anyhow!("{}", e))?;
    info!("Started session.");

    let (writer, reader) = open_stream(&session).await?;

    let rpc_network = VatNetwork::new(
        Box::pin(reader),
        Box::pin(writer),
        Side::Client,
        Default::default(),
    );
    let mut rpc_system = RpcSystem::new(Box::new(rpc_network), None);

    let world_server: Client = rpc_system.bootstrap(Side::Server);

    tokio::task::spawn_local(
        async move {
            match rpc_system.await {
                Ok(_) => debug!("Graceful exit."),
                Err(e) => error!("{}", e),
            };
        }
        .instrument(info_span!("RPC")),
    );
    info!("Created world server RPC.");

    join_instance(&world_server, record_id).await?;

    while let Some(action) = action_receiver.recv().await {
        debug!("Handling action: {:?}", action);

        match action {
            InstanceAction::Close => break,
        }
    }

    Ok(())
}

async fn open_stream<T: Session>(session: &T) -> Result<(WriteCompat<T>, ReadCompat<T>)> {
    let (send, recv) = session
        .open_bi()
        .await
        .map_err(|e| anyhow!("{}", e))?
        .wait_bi()
        .await
        .map_err(|e| anyhow!("{}", e))?;

    let writer = WriteCompat::<T>::new(send);
    let reader = ReadCompat::<T>::new(recv);

    Ok((writer, reader))
}

async fn join_instance(world_server: &Client, record_id: String) -> Result<()> {
    let mut request = world_server.join_instance_request();
    request.get().init_instance().set_record_id(record_id);

    let reply = request.send().promise.await.map_err(|e| anyhow!("{}", e))?;
    let response = reply
        .get()
        .map_err(|e| anyhow!("{}", e))?
        .get_response()
        .map_err(|e| anyhow!("{}", e))?;

    match response.which().unwrap() {
        Which::Success(s) => {
            let ok = s.unwrap().get_ok();
            info!("Join success: {}", ok)
        }
        Which::Error(e) => {
            let e = e.unwrap().to_str().unwrap();
            error!("Join error: {}", e);
        }
    };

    Ok(())
}
