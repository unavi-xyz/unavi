//! `wired:perception/perception`.
//!
//! Exported as `perception*` because `wasm-bindgen` names are shared by every
//! package on [`Runtime`].

use std::rc::Rc;

use unavi_policy::trust::Trust;
use unavi_portal::portal::PortalState;
use wasm_bindgen::prelude::*;

use super::{
    Runtime,
    convert,
    inline_promise as future_to_promise,
};
use crate::host::perception::{
    self,
    Portal,
};

const fn state_name(state: PortalState) -> &'static str {
    match state {
        PortalState::Closed => "closed",
        PortalState::Loading => "loading",
        PortalState::Open => "open",
    }
}

const fn rung_name(trust: Trust) -> &'static str {
    match trust {
        Trust::Blocked => "blocked",
        Trust::Anonymous => "anonymous",
        Trust::Guest => "guest",
        Trust::Trusted => "trusted",
        Trust::Myself => "myself",
    }
}

fn record(fields: &[(&str, JsValue)]) -> JsValue {
    let obj = js_sys::Object::new();
    for (name, value) in fields {
        js_sys::Reflect::set(&obj, &(*name).into(), value).ok();
    }
    obj.into()
}

fn wit_portal(portal: &Portal) -> JsValue {
    record(&[
        ("prim", convert::wit_prim_ref((portal.doc, portal.prim))),
        ("transform", convert::wit_transform(portal.transform)),
        ("width", portal.width.into()),
        ("height", portal.height.into()),
        (
            "destination",
            portal
                .destination
                .map_or(JsValue::UNDEFINED, convert::wit_doc_id),
        ),
        ("linked", portal.linked.into()),
        ("state", state_name(portal.state).into()),
        ("hue", convert::wit_color(portal.hue)),
    ])
}

fn optional_string(value: Option<String>) -> JsValue {
    value.map_or(JsValue::UNDEFINED, JsValue::from)
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "perceptionPortals")]
    pub fn perception_portals(&self, radius: f32) -> Result<Vec<JsValue>, JsValue> {
        perception::portals(&self.host.borrow(), radius)
            .map(|portals| portals.iter().map(wit_portal).collect())
            .map_err(convert::raise)
    }

    #[wasm_bindgen(js_name = "perceptionMeta")]
    pub fn perception_meta(&self, document: JsValue) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        let id = match convert::doc_id(document) {
            Ok(id) => id,
            Err(err) => return js_sys::Promise::reject(&convert::raise(err)),
        };
        future_to_promise(async move {
            let host = host.borrow();
            perception::meta(&host, id)
                .await
                .map(|meta| {
                    meta.map_or(JsValue::UNDEFINED, |meta| {
                        record(&[
                            ("name", optional_string(meta.name)),
                            ("description", optional_string(meta.description)),
                        ])
                    })
                })
                .map_err(convert::raise)
        })
    }

    #[wasm_bindgen(js_name = "perceptionRung")]
    pub fn perception_rung(&self, user: &str) -> Result<JsValue, JsValue> {
        perception::rung(&self.host.borrow(), user)
            .map(|trust| rung_name(trust).into())
            .map_err(convert::raise)
    }
}
