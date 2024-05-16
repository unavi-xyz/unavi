use capnp::{capability::Promise, Error};
use tracing::info;
use wired_world::world_server_capnp::world_server::{
    JoinInstanceParams, JoinInstanceResults, ListPlayersParams, ListPlayersResults, Server,
};

pub struct WorldServer {}

impl Server for WorldServer {
    fn join_instance(
        &mut self,
        _: JoinInstanceParams,
        _: JoinInstanceResults,
    ) -> Promise<(), Error> {
        info!("join_instance");
        Promise::ok(())
    }

    fn list_players(&mut self, _: ListPlayersParams, _: ListPlayersResults) -> Promise<(), Error> {
        info!("list_players");
        Promise::ok(())
    }
}
