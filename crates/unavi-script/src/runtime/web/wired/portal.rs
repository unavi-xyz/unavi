use std::sync::Arc;

use wasm_bindgen::{JsValue, prelude::*};

use crate::runtime::{
    Runtime,
    shared::{
        self, Api,
        wired::portal::{PortalDestination, PortalParams, PortalTransform},
    },
};

fn portal_dest_to_js(dest: PortalDestination) -> JsValue {
    let obj = js_sys::Object::new();
    let space: js_sys::Uint8Array = dest.space.as_slice().into();
    js_sys::Reflect::set(&obj, &"space".into(), &space.into()).ok();
    let portal: JsValue = dest.portal.map_or(JsValue::NULL, Into::into);
    js_sys::Reflect::set(&obj, &"portal".into(), &portal).ok();
    obj.into()
}

#[wasm_bindgen]
pub struct PortalHandle {
    rep: u32,
    api: Arc<Api>,
}

impl PortalHandle {
    pub const fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }
}

impl Drop for PortalHandle {
    fn drop(&mut self) {
        if self.rep != u32::MAX {
            let _ = shared::wired::portal::on_drop(&self.api, self.rep);
        }
    }
}

fn js_to_portal_params(value: &JsValue) -> PortalParams {
    let get = |obj: &JsValue, k: &str| {
        js_sys::Reflect::get(obj, &k.into())
            .ok()
            .unwrap_or_default()
    };
    let f32_at = |obj: &JsValue, k: &str, d: f32| {
        js_sys::Reflect::get(obj, &k.into())
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(d as f64) as f32
    };

    let dest_js = get(value, "destination");
    let space = js_sys::Uint8Array::new(&get(&dest_js, "space")).to_vec();
    let portal = js_sys::Reflect::get(&dest_js, &"portal".into())
        .ok()
        .and_then(|v| v.as_string());

    let size_js = get(value, "size");
    let size = [f32_at(&size_js, "x", 0.0), f32_at(&size_js, "y", 0.0)];

    let tf_js = get(value, "transform");
    let tr = get(&tf_js, "translation");
    let ro = get(&tf_js, "rotation");
    let sc = get(&tf_js, "scale");
    let transform = PortalTransform {
        translation: [f32_at(&tr, "x", 0.0), f32_at(&tr, "y", 0.0), f32_at(&tr, "z", 0.0)],
        rotation: [
            f32_at(&ro, "x", 0.0),
            f32_at(&ro, "y", 0.0),
            f32_at(&ro, "z", 0.0),
            f32_at(&ro, "w", 1.0),
        ],
        scale: [f32_at(&sc, "x", 1.0), f32_at(&sc, "y", 1.0), f32_at(&sc, "z", 1.0)],
    };

    PortalParams {
        destination: PortalDestination { space, portal },
        size,
        transform,
    }
}

#[wasm_bindgen]
impl PortalHandle {
    pub fn close(&self) -> Result<(), String> {
        shared::wired::portal::close(&self.api, self.rep).map_err(|e| e.to_string())
    }

    pub fn destination(&self) -> JsValue {
        shared::wired::portal::destination(&self.api, self.rep)
            .map_or(JsValue::NULL, portal_dest_to_js)
    }

    pub fn id(&self) -> String {
        shared::wired::portal::id(&self.api, self.rep).unwrap_or_default()
    }
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredPortalClass")]
    pub fn wired_portal_class(&self) -> JsValue {
        let handle = PortalHandle::new(u32::MAX, Arc::clone(&self.api));
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredPortalListPortals")]
    pub fn wired_portal_list_portals(&self) -> JsValue {
        let reps = shared::wired::portal::list_portals(&self.api).unwrap_or_default();
        reps.into_iter()
            .map(|rep| JsValue::from(PortalHandle::new(rep, Arc::clone(&self.api))))
            .collect::<js_sys::Array>()
            .into()
    }

    #[wasm_bindgen(js_name = "wiredPortalOpenPortal")]
    pub fn wired_portal_open_portal(&self, params: JsValue) -> Result<PortalHandle, String> {
        let rep = shared::wired::portal::open_portal(&self.api, js_to_portal_params(&params))
            .map_err(|e| e.to_string())?;
        Ok(PortalHandle::new(rep, Arc::clone(&self.api)))
    }
}
