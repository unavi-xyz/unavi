use wasm_bindgen::prelude::*;

use super::scene::util::opt_rep;
use crate::{
    permissions::ApiName,
    runtime::{
        Runtime,
        shared,
    },
};

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredPortalOpen")]
    pub async fn wired_portal_open(
        &self,
        prim: JsValue,
        target_space: Vec<u8>,
    ) -> Result<(), String> {
        self.api
            .require(ApiName::Portal)
            .map_err(|e| e.to_string())?;
        let rep = opt_rep(&prim).ok_or_else(|| "invalid prim handle".to_string())?;
        shared::wired::portal::open(&self.api, rep, target_space)
            .await
            .map_err(|e| e.to_string())
    }

    #[wasm_bindgen(js_name = "wiredPortalTravel")]
    pub async fn wired_portal_travel(&self, target_space: Vec<u8>) -> Result<(), String> {
        self.api
            .require(ApiName::System)
            .map_err(|e| e.to_string())?;
        shared::wired::portal::travel(&self.api, target_space)
            .await
            .map_err(|e| e.to_string())
    }
}
