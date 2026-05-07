use std::sync::Arc;

use wasm_bindgen::prelude::*;

use crate::runtime::{
    Runtime,
    shared::{
        self, Api,
        wired::portal::{PortalDestination, PortalParams, PortalTransform},
    },
};

#[wasm_bindgen]
pub struct PortalHandle {
    rep: u32,
    api: Arc<Api>,
}

impl PortalHandle {
    pub fn new(rep: u32, api: Arc<Api>) -> Self {
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
    let get = |k: &str| {
        js_sys::Reflect::get(value, &k.into())
            .ok()
            .unwrap_or_default()
    };

    let dest_js = get("destination");
    let space = js_sys::Uint8Array::new(
        &js_sys::Reflect::get(&dest_js, &"space".into()).unwrap_or_default(),
    )
    .to_vec();
    let portal = js_sys::Reflect::get(&dest_js, &"portal".into())
        .ok()
        .and_then(|v| v.as_string());

    let size_js = js_sys::Array::from(&get("size"));
    let size = [
        size_js.get(0).as_f64().unwrap_or(0.0) as f32,
        size_js.get(1).as_f64().unwrap_or(0.0) as f32,
    ];

    let tf_js = get("transform");
    let tr = js_sys::Array::from(
        &js_sys::Reflect::get(&tf_js, &"translation".into()).unwrap_or_default(),
    );
    let ro =
        js_sys::Array::from(&js_sys::Reflect::get(&tf_js, &"rotation".into()).unwrap_or_default());
    let sc =
        js_sys::Array::from(&js_sys::Reflect::get(&tf_js, &"scale".into()).unwrap_or_default());
    let transform = PortalTransform {
        translation: [
            tr.get(0).as_f64().unwrap_or(0.0) as f32,
            tr.get(1).as_f64().unwrap_or(0.0) as f32,
            tr.get(2).as_f64().unwrap_or(0.0) as f32,
        ],
        rotation: [
            ro.get(0).as_f64().unwrap_or(0.0) as f32,
            ro.get(1).as_f64().unwrap_or(0.0) as f32,
            ro.get(2).as_f64().unwrap_or(0.0) as f32,
            ro.get(3).as_f64().unwrap_or(1.0) as f32,
        ],
        scale: [
            sc.get(0).as_f64().unwrap_or(1.0) as f32,
            sc.get(1).as_f64().unwrap_or(1.0) as f32,
            sc.get(2).as_f64().unwrap_or(1.0) as f32,
        ],
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
        let Ok(dest) = shared::wired::portal::destination(&self.api, self.rep) else {
            return JsValue::NULL;
        };
        let obj = js_sys::Object::new();
        let space: js_sys::Uint8Array = dest.space.as_slice().into();
        js_sys::Reflect::set(&obj, &"space".into(), &space.into()).ok();
        match dest.portal {
            Some(p) => js_sys::Reflect::set(&obj, &"portal".into(), &p.into()).ok(),
            None => js_sys::Reflect::set(&obj, &"portal".into(), &JsValue::NULL).ok(),
        };
        obj.into()
    }

    pub fn id(&self) -> String {
        shared::wired::portal::id(&self.api, self.rep).unwrap_or_default()
    }
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredPortalClass")]
    pub fn wired_portal_class(&self) -> JsValue {
        let handle = PortalHandle::new(u32::MAX, self.api.clone());
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
        Ok(PortalHandle::new(rep, self.api.clone()))
    }
}
