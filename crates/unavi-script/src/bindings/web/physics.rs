//! `wired:physics/simulation`.

use std::rc::Rc;

use wasm_bindgen::prelude::*;

use super::{
    Runtime,
    convert,
    scene::DocumentHandle,
};
use crate::{
    error::ScriptError,
    host::physics,
};

fn ray_filter(value: &JsValue) -> Result<physics::RayFilter, ScriptError> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct WireFilter {
        exclude_local_agent: bool,
        exclude_documents:   Vec<(u64, u64, u64, u64)>,
    }
    let f: WireFilter = convert::from_js(value.clone())?;
    if f.exclude_documents.len() > physics::MAX_EXCLUDED_DOCUMENTS {
        return Err(ScriptError::invalid(
            "a filter excludes at most 64 documents",
        ));
    }
    Ok(physics::RayFilter {
        exclude_local_agent: f.exclude_local_agent,
        exclude_documents:   f
            .exclude_documents
            .into_iter()
            .map(convert::doc_id_from_words)
            .collect(),
    })
}

fn wit_ray_hit(hit: physics::RayHit) -> JsValue {
    let obj = js_sys::Object::new();
    let set = |key: &str, value: &JsValue| {
        js_sys::Reflect::set(&obj, &JsValue::from_str(key), value).ok();
    };
    set("target", &convert::wit_prim_ref((hit.document, hit.prim)));
    set("point", &convert::wit_vec3(hit.point));
    set("normal", &convert::wit_vec3(hit.normal));
    set("distance", &JsValue::from_f64(f64::from(hit.distance)));
    obj.into()
}

fn wit_body_velocity(v: physics::BodyVelocity) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"linear".into(), &convert::wit_vec3(v.linear)).ok();
    js_sys::Reflect::set(&obj, &"angular".into(), &convert::wit_vec3(v.angular)).ok();
    obj.into()
}

#[wasm_bindgen]
impl Runtime {
    pub fn raycast(&self, ray: JsValue, max_distance: f32, filter: JsValue) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        let parsed = convert::ray(ray).and_then(|ray| Ok((ray, ray_filter(&filter)?)));
        let ((origin, direction), filter) = match parsed {
            Ok(v) => v,
            Err(err) => return js_sys::Promise::reject(&convert::raise(err)),
        };
        wasm_bindgen_futures::future_to_promise(async move {
            let host = host.borrow();
            physics::raycast(&host, origin, direction, max_distance, filter)
                .await
                .map(|hit| hit.map_or(JsValue::UNDEFINED, wit_ray_hit))
                .map_err(convert::raise)
        })
    }

    pub fn velocity(&self, document: &DocumentHandle, prim: JsValue) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        let doc = document.rep();
        let prim = match convert::prim_id(prim) {
            Ok(p) => p,
            Err(err) => return js_sys::Promise::reject(&convert::raise(err)),
        };
        wasm_bindgen_futures::future_to_promise(async move {
            let host = host.borrow();
            physics::velocity(&host, doc, prim)
                .await
                .map(wit_body_velocity)
                .map_err(convert::raise)
        })
    }

    #[wasm_bindgen(js_name = "setVelocity")]
    pub fn set_velocity(
        &self,
        document: &DocumentHandle,
        prim: JsValue,
        linear: JsValue,
        angular: JsValue,
    ) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        let doc = document.rep();
        let parsed = (|| -> Result<_, ScriptError> {
            Ok((
                convert::prim_id(prim)?,
                opt_vec3(linear)?,
                opt_vec3(angular)?,
            ))
        })();
        let (prim, linear, angular) = match parsed {
            Ok(v) => v,
            Err(err) => return js_sys::Promise::reject(&convert::raise(err)),
        };
        wasm_bindgen_futures::future_to_promise(async move {
            let host = host.borrow();
            physics::set_velocity(&host, doc, prim, linear, angular)
                .await
                .map(|()| JsValue::UNDEFINED)
                .map_err(convert::raise)
        })
    }

    #[wasm_bindgen(js_name = "setForce")]
    pub fn set_force(
        &self,
        document: &DocumentHandle,
        prim: JsValue,
        force: JsValue,
    ) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        let doc = document.rep();
        let parsed = (|| -> Result<_, ScriptError> {
            Ok((convert::prim_id(prim)?, convert::vec3(force)?))
        })();
        let (prim, force) = match parsed {
            Ok(v) => v,
            Err(err) => return js_sys::Promise::reject(&convert::raise(err)),
        };
        wasm_bindgen_futures::future_to_promise(async move {
            let host = host.borrow();
            physics::set_force(&host, doc, prim, force)
                .await
                .map(|()| JsValue::UNDEFINED)
                .map_err(convert::raise)
        })
    }
}

fn opt_vec3(value: JsValue) -> Result<Option<bevy::math::Vec3>, ScriptError> {
    if value.is_undefined() {
        Ok(None)
    } else {
        convert::vec3(value).map(Some)
    }
}
