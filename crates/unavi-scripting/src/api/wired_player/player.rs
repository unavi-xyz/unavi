use wasm_bridge::component::Resource;

use super::wired::player::api::{HostPlayer, Skeleton};

pub struct Player;

impl HostPlayer for Player {
    fn skeleton(&mut self, _self_: Resource<Player>) -> wasm_bridge::Result<Skeleton> {
        todo!();
    }

    fn drop(&mut self, _rep: Resource<Player>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}
