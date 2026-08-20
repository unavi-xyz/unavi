use unavi_policy::document::ApiName;
use wasm_bindgen::prelude::*;

use super::{
    raise,
    scene::util::opt_rep,
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
        prim: JsValue,
        target_space: Vec<u8>,
    ) -> Result<(), JsValue> {
        self.api.require(ApiName::Portal).map_err(raise)?;
        let rep = opt_rep(&prim).ok_or_else(|| "invalid prim handle".to_string())?;
        shared::wired::portal::open(&self.api, rep, target_space)
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "wiredPortalTravel")]
    pub async fn wired_portal_travel(&self, target_space: Vec<u8>) -> Result<(), JsValue> {
        self.api.require(ApiName::Travel).map_err(raise)?;
        shared::wired::portal::travel(&self.api, target_space)
            .await
            .map_err(raise)
    }
}
