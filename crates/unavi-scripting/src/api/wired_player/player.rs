use std::cell::Cell;

use wasm_bridge::component::Resource;

use crate::{
    api::utils::{RefCount, RefCountCell, RefResource},
    state::StoreState,
};

use super::wired::player::api::{HostPlayer, Skeleton};

pub struct Player {
    ref_count: RefCountCell,
}

impl RefCount for Player {
    fn ref_count(&self) -> &Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for Player {}

impl HostPlayer for StoreState {
    fn skeleton(&mut self, _self_: Resource<Player>) -> wasm_bridge::Result<Skeleton> {
        todo!();
    }

    fn drop(&mut self, _rep: Resource<Player>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}
