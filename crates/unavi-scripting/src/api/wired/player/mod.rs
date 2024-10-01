use anyhow::Result;
use wasm_bridge::component::{Linker, Resource};

use crate::{api::utils::RefResource, data::ScriptData};

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
            "wired:player/api/player": super::player::Player,
        }
    });

    pub use wired::player::*;
}

pub fn add_to_linker(linker: &mut Linker<ScriptData>) -> Result<()> {
    bindings::api::add_to_linker(linker, |s| s)?;
    Ok(())
}

pub struct WiredPlayer {
    pub local_player: Resource<player::Player>,
}

impl bindings::api::Host for ScriptData {
    fn list_players(&mut self) -> wasm_bridge::Result<Vec<Resource<player::Player>>> {
        Ok(Vec::default())
    }

    fn local_player(&mut self) -> wasm_bridge::Result<Resource<player::Player>> {
        let p = player::Player::from_res(
            &self.api.wired_player.as_ref().unwrap().local_player,
            &self.table,
        )?;
        Ok(p)
    }
}
