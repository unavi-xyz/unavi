use capnp::{capability::Promise, Error};
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
        Promise::ok(())
    }

    fn list_players(&mut self, _: ListPlayersParams, _: ListPlayersResults) -> Promise<(), Error> {
        Promise::ok(())
    }
}
