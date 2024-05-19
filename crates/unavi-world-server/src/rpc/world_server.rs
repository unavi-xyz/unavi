use std::sync::Arc;

use capnp::{capability::Promise, Error};
use capnp_rpc::pry;
use dwn::{
    store::{DataStore, MessageStore},
    DWN,
};
use tracing::info;
use wired_world::world_server_capnp::world_server::{
    JoinInstanceParams, JoinInstanceResults, ListPlayersParams, ListPlayersResults, Server,
};

pub struct WorldServer<D: DataStore, M: MessageStore> {
    pub connection_id: usize,
    pub dwn: Arc<DWN<D, M>>,
}

impl<D: DataStore, M: MessageStore> Server for WorldServer<D, M> {
    fn join_instance(
        &mut self,
        params: JoinInstanceParams,
        _results: JoinInstanceResults,
    ) -> Promise<(), Error> {
        let params = pry!(params.get());
        let instance = pry!(params.get_instance());
        let record_id = pry!(pry!(instance.get_record_id()).to_str());

        info!("Request to join instance: {}", record_id);

        // results.get().init_response().set_success();

        Promise::ok(())
    }

    fn list_players(&mut self, _: ListPlayersParams, _: ListPlayersResults) -> Promise<(), Error> {
        info!("list_players");
        Promise::ok(())
    }
}
