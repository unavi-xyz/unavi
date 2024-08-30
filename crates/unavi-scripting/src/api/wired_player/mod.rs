use anyhow::Result;
use wasm_bridge::component::{Linker, Resource};

use crate::state::StoreState;

use super::utils::RefResource;

pub(crate) mod player;
pub mod systems;

wasm_bridge::component::bindgen!({
    path: "../../wired-protocol/spatial/wit/wired-player",
    world: "host",
    with: {
        "wired:player/api/player": player::Player,
        "wired:scene/node/node": crate::api::wired_scene::gltf::node::NodeRes,
    }
});

pub fn add_to_linker(linker: &mut Linker<StoreState>) -> Result<()> {
    wired::player::api::add_to_linker(linker, |s| s)?;
    Ok(())
}

impl wired::player::api::Host for StoreState {
    fn list_players(&mut self) -> wasm_bridge::Result<Vec<Resource<player::Player>>> {
        Ok(Vec::default())
    }

    fn local_player(&mut self) -> wasm_bridge::Result<Resource<player::Player>> {
        let p = player::Player::from_res(&self.local_player, &self.table)?;
        Ok(p)
    }
}
