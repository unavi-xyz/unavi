use unavi_policy::document::ApiName;
use wasm_bindgen::prelude::*;

use crate::runtime::{
    Runtime,
    shared,
    web::wired::{
        raise,
        scene::{
            prim::PrimHandle,
            util::{
                js_to_vec3,
                vec3_to_js,
            },
        },
    },
};

fn ray_hit_to_js(hit: &shared::wired::physics::RayHit) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(
        &obj,
        &"document".into(),
        &js_sys::Uint8Array::from(hit.document.as_slice()).into(),
    )
    .ok();
    js_sys::Reflect::set(&obj, &"prim".into(), &JsValue::from_str(&hit.prim)).ok();
    js_sys::Reflect::set(
        &obj,
        &"point".into(),
        &vec3_to_js(hit.point[0], hit.point[1], hit.point[2]),
    )
    .ok();
    js_sys::Reflect::set(
        &obj,
        &"normal".into(),
        &vec3_to_js(hit.normal[0], hit.normal[1], hit.normal[2]),
    )
    .ok();
    js_sys::Reflect::set(&obj, &"distance".into(), &hit.distance.into()).ok();
    obj.into()
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredPhysicsRaycast")]
    pub async fn wired_physics_raycast(
        &self,
        origin: JsValue,
        dir: JsValue,
        max_dist: f32,
    ) -> Result<JsValue, JsValue> {
        self.api.require(ApiName::Physics).map_err(raise)?;
        let origin = js_to_vec3(&origin, [0.0; 3]);
        let dir = js_to_vec3(&dir, [0.0; 3]);
        let hit = shared::wired::physics::raycast(&self.api, origin, dir, max_dist)
            .await
            .map_err(raise)?;
        Ok(hit.map_or(JsValue::UNDEFINED, |h| ray_hit_to_js(&h)))
    }

    #[wasm_bindgen(js_name = "wiredPhysicsGetLinearVelocity")]
    pub async fn wired_physics_get_linear_velocity(
        &self,
        prim: &PrimHandle,
    ) -> Result<JsValue, JsValue> {
        self.api.require(ApiName::Physics).map_err(raise)?;
        let v = shared::wired::physics::get_linear_velocity(&self.api, prim.rep())
            .await
            .map_err(raise)?;
        Ok(vec3_to_js(v[0], v[1], v[2]))
    }

    #[wasm_bindgen(js_name = "wiredPhysicsApplyForce")]
    pub async fn wired_physics_apply_force(
        &self,
        prim: &PrimHandle,
        force: JsValue,
    ) -> Result<(), JsValue> {
        self.api.require(ApiName::Physics).map_err(raise)?;
        let f = js_to_vec3(&force, [0.0; 3]);
        shared::wired::physics::apply_force(&self.api, prim.rep(), f)
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "wiredPhysicsSetLinearVelocity")]
    pub async fn wired_physics_set_linear_velocity(
        &self,
        prim: &PrimHandle,
        v: JsValue,
    ) -> Result<(), JsValue> {
        self.api.require(ApiName::Physics).map_err(raise)?;
        let v = js_to_vec3(&v, [0.0; 3]);
        shared::wired::physics::set_linear_velocity(&self.api, prim.rep(), v)
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "wiredPhysicsSetAngularVelocity")]
    pub async fn wired_physics_set_angular_velocity(
        &self,
        prim: &PrimHandle,
        v: JsValue,
    ) -> Result<(), JsValue> {
        self.api.require(ApiName::Physics).map_err(raise)?;
        let v = js_to_vec3(&v, [0.0; 3]);
        shared::wired::physics::set_angular_velocity(&self.api, prim.rep(), v)
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "wiredPhysicsClaimAuthority")]
    pub fn wired_physics_claim_authority(&self, doc: Vec<u8>) -> Result<(), JsValue> {
        self.api.require(ApiName::Physics).map_err(raise)?;
        shared::wired::physics::claim_authority(&self.api, doc).map_err(raise)
    }

    #[wasm_bindgen(js_name = "wiredPhysicsReleaseAuthority")]
    pub fn wired_physics_release_authority(&self, doc: Vec<u8>) -> Result<(), JsValue> {
        self.api.require(ApiName::Physics).map_err(raise)?;
        shared::wired::physics::release_authority(&self.api, doc).map_err(raise)
    }
}
