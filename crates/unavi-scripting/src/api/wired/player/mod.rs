use anyhow::Result;
use wasm_bridge::component::{Linker, Resource};

use crate::{api::utils::RefResource, data::StoreData};

#[allow(clippy::module_inception)]
mod player;
pub mod systems;

pub mod bindings {
    pub use super::player::Player;

    wasm_bridge::component::bindgen!({
        path: "../../wired-protocol/spatial/wit/wired-player",
        world: "host",
        with: {
            "wired:math": crate::api::wired::math::bindings,
            "wired:scene": crate::api::wired::scene::bindings,
            "wired:player/api/player": Player,
        }
    });

    pub use wired::player::*;
}

pub fn add_to_linker(linker: &mut Linker<StoreData>) -> Result<()> {
    bindings::api::add_to_linker(linker, |s| s)?;
    Ok(())
}

impl bindings::api::Host for StoreData {
    fn list_players(&mut self) -> wasm_bridge::Result<Vec<Resource<player::Player>>> {
        Ok(Vec::default())
    }

    fn local_player(&mut self) -> wasm_bridge::Result<Resource<player::Player>> {
        let p = player::Player::from_res(&self.local_player, &self.table)?;
        Ok(p)
    }
}
