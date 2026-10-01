use std::sync::Arc;

use bevy_async::task;
use hsd::attributes::xform::XformAttr;
use unavi_policy::permissions::HostApi;
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
    web::wired::{
        malformed,
        raise,
    },
};

/// `list<tuple<prim-id, string>>`, which `jco` lowers as an array of pairs.
fn js_to_commit_props(v: &JsValue) -> Result<Vec<(String, String)>, String> {
    js_sys::Array::from(v)
        .iter()
        .map(|entry| {
            let pair = js_sys::Array::from(&entry);
            match (pair.get(0).as_string(), pair.get(1).as_string()) {
                (Some(prim), Some(prop)) => Ok((prim, prop)),
                _ => Err("a commit entry is a [prim-id, string] pair".to_string()),
            }
        })
        .collect()
}

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
            task::spawn(async move {
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

    pub async fn commit(&self, props: JsValue) -> Result<(), JsValue> {
        self.api.require(HostApi::Commit).map_err(raise)?;
        let props = js_to_commit_props(&props).map_err(malformed)?;
        shared::wired::scene::document::commit(&self.api, self.rep, props)
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "setAnchor")]
    pub async fn set_anchor(&self, target: JsValue) -> Result<(), JsValue> {
        shared::wired::scene::document::set_anchor(&self.api, self.rep, opt_rep(&target))
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "setOffset")]
    pub async fn set_offset(&self, value: JsValue) -> Result<(), JsValue> {
        let x = js_to_xform(&value).ok_or_else(|| malformed("an offset is a transform".into()))?;
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
