use wasm_bindgen::prelude::*;

use super::scene::prim::PrimHandle;
use crate::runtime::{
    Runtime,
    shared,
};

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredPortalOpenPortal")]
    pub async fn wired_portal_open_portal(
        &self,
        prim: &PrimHandle,
        space: Vec<u8>,
    ) -> Result<(), String> {
        let space: [u8; 32] = space
            .as_slice()
            .try_into()
            .map_err(|_| "space id must be 32 bytes".to_string())?;
        shared::wired::portal::open_portal(&self.api, prim.rep(), space)
            .await
            .map_err(|e| e.to_string())
    }
}
