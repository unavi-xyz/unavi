use std::sync::Arc;

use hsd::attributes::xform::XformAttr;
use unavi_util::async_task::spawn_async_task;
use wasm_bindgen::prelude::*;

use super::{
    prim::PrimHandle,
    util::{
        js_to_xform,
        opt_rep,
        xform_to_js,
    },
};
use crate::runtime::{
    shared::{
        self,
        Api,
        wired::scene::document::XformValue,
    },
    web::wired::raise,
};

#[wasm_bindgen]
pub struct DocHandle {
    rep: u32,
    api: Arc<Api>,
}

impl DocHandle {
    pub const fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }
}

impl Drop for DocHandle {
    fn drop(&mut self) {
        if self.rep != u32::MAX {
            let api = Arc::clone(&self.api);
            let rep = self.rep;
            spawn_async_task(async move {
                let _ = shared::wired::scene::document::on_drop(&api, rep).await;
            });
        }
    }
}

#[wasm_bindgen]
impl DocHandle {
    pub async fn id(&self) -> Vec<u8> {
        shared::wired::scene::document::id(&self.api, self.rep)
            .await
            .unwrap_or_default()
    }

    #[wasm_bindgen(js_name = "clone")]
    pub async fn clone_doc(&self) -> Option<Self> {
        let rep = shared::wired::scene::document::clone(&self.api, self.rep)
            .await
            .ok()?;
        Some(Self::new(rep, Arc::clone(&self.api)))
    }

    pub async fn roots(&self) -> js_sys::Array {
        let Ok(reps) = shared::wired::scene::document::roots(&self.api, self.rep).await else {
            return js_sys::Array::new();
        };
        reps.into_iter()
            .map(|rep| JsValue::from(PrimHandle::new(rep, Arc::clone(&self.api))))
            .collect()
    }

    pub async fn prims(&self) -> js_sys::Array {
        let Ok(reps) = shared::wired::scene::document::prims(&self.api, self.rep).await else {
            return js_sys::Array::new();
        };
        reps.into_iter()
            .map(|rep| JsValue::from(PrimHandle::new(rep, Arc::clone(&self.api))))
            .collect()
    }

    #[wasm_bindgen(js_name = "getPrim")]
    pub async fn get_prim(&self, id: String) -> Option<PrimHandle> {
        let rep = shared::wired::scene::document::get_prim(&self.api, self.rep, id)
            .await
            .ok()??;
        Some(PrimHandle::new(rep, Arc::clone(&self.api)))
    }

    #[wasm_bindgen(js_name = "createPrim")]
    pub async fn create_prim(&self) -> Result<PrimHandle, JsValue> {
        let rep = shared::wired::scene::document::create_prim(&self.api, self.rep)
            .await
            .map_err(raise)?;
        Ok(PrimHandle::new(rep, Arc::clone(&self.api)))
    }

    #[wasm_bindgen(js_name = "removePrim")]
    pub async fn remove_prim(&self, value: &PrimHandle) -> Result<(), JsValue> {
        shared::wired::scene::document::remove_prim(&self.api, value.rep())
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "offsetTo")]
    pub async fn offset_to(&self, other: &Self) -> JsValue {
        match shared::wired::scene::document::offset_to(&self.api, self.rep, other.rep).await {
            Ok(Some(x)) => xform_to_js(&XformAttr {
                translation: x.translation,
                rotation:    x.rotation,
                scale:       x.scale,
            }),
            _ => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = "setAnchor")]
    pub async fn set_anchor(&self, target: JsValue) -> Result<(), JsValue> {
        shared::wired::scene::document::set_anchor(&self.api, self.rep, opt_rep(&target))
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "setOffset")]
    pub async fn set_offset(&self, value: JsValue) -> Result<(), JsValue> {
        let x = js_to_xform(&value).unwrap_or_default();
        shared::wired::scene::document::set_offset(
            &self.api,
            self.rep,
            XformValue {
                translation: x.translation,
                rotation:    x.rotation,
                scale:       x.scale,
            },
        )
        .await
        .map_err(raise)
    }
}
