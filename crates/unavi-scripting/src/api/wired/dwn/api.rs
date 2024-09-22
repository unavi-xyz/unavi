use wasm_bridge::component::Resource;

use crate::data::StoreData;

use super::bindings::dwn::Dwn;

impl super::bindings::api::Host for StoreData {
    fn local_dwn(&mut self) -> wasm_bridge::Result<Resource<Dwn>> {
        todo!();
    }

    fn world_host_dwn(&mut self) -> wasm_bridge::Result<Resource<Dwn>> {
        todo!();
    }
}
