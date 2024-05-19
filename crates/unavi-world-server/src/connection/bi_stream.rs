use std::sync::Arc;

use capnp_rpc::{rpc_twoparty_capnp::Side, twoparty::VatNetwork, RpcSystem};
use dwn::{
    store::{DataStore, MessageStore},
    DWN,
};
use tracing::{debug, error};
use xwt_futures_io::{read::ReadCompat, write::WriteCompat};
use xwt_wtransport::{Connection, RecvStream, SendStream};

use crate::rpc::world_server::WorldServer;

pub async fn handle_bi_stream<D: DataStore + 'static, M: MessageStore + 'static>(
    connection_id: usize,
    dwn: Arc<DWN<D, M>>,
    (send, recv): (SendStream, RecvStream),
) {
    let world_server_client: wired_world::world_server_capnp::world_server::Client =
        capnp_rpc::new_client(WorldServer { connection_id, dwn });

    let reader = ReadCompat::<Connection>::new(recv);
    let writer = WriteCompat::<Connection>::new(send);

    let network = VatNetwork::new(reader, writer, Side::Server, Default::default());
    let rpc_system = RpcSystem::new(Box::new(network), Some(world_server_client.clone().client));

    if let Err(e) = rpc_system.await {
        error!("{}", e);
    };

    debug!("Stream closed.");
}
