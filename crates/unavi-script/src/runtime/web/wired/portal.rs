use unavi_policy::permissions::HostApi;
use wasm_bindgen::prelude::*;

use super::{
    raise,
    scene::prim::PrimHandle,
};
use crate::runtime::{
    Runtime,
    shared,
};

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredPortalOpen")]
    pub async fn wired_portal_open(
        &self,
        prim: &PrimHandle,
        target_space: Vec<u8>,
    ) -> Result<(), JsValue> {
        self.api.require(HostApi::Portal).map_err(raise)?;
        shared::wired::portal::open(&self.api, prim.rep(), target_space)
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "wiredPortalPair")]
    pub async fn wired_portal_pair(
        &self,
        prim: &PrimHandle,
        source_space: Vec<u8>,
        link: Vec<u8>,
    ) -> Result<(), JsValue> {
        self.api.require(HostApi::Portal).map_err(raise)?;
        shared::wired::portal::pair(&self.api, prim.rep(), source_space, link)
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "wiredPortalTravel")]
    pub async fn wired_portal_travel(&self, target_space: Vec<u8>) -> Result<(), JsValue> {
        self.api.require(HostApi::Travel).map_err(raise)?;
        shared::wired::portal::travel(&self.api, target_space)
            .await
            .map_err(raise)
    }
}
