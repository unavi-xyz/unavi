use anyhow::Result;
use wasm_bridge::component::{Linker, Resource};

use crate::data::ScriptData;

#[allow(clippy::module_inception)]
pub mod player;
pub mod systems;

pub mod bindings {
    wasm_bridge::component::bindgen!({
        path: "../../wired-protocol/spatial/wit/wired-player",
        world: "host",
        with: {
            "wired:math": crate::api::wired::math::bindings,
            "wired:scene": crate::api::wired::scene::bindings,
            "wired:player/api/player": super::player::PlayerRes,
        }
    });

    pub use wired::player::*;
}

pub fn add_to_linker(linker: &mut Linker<ScriptData>) -> Result<()> {
    bindings::api::add_to_linker(linker, |s| s)?;
    Ok(())
}

pub struct WiredPlayer {
    pub local_player: player::PlayerRes,
}

impl bindings::api::Host for ScriptData {
    fn list_players(&mut self) -> wasm_bridge::Result<Vec<Resource<player::PlayerRes>>> {
        Ok(Vec::default())
    }

    fn local_player(&mut self) -> wasm_bridge::Result<Resource<player::PlayerRes>> {
        let res = self
            .table
            .push(self.api.wired_player.as_ref().unwrap().local_player.clone())?;
        Ok(res)
    }
}
