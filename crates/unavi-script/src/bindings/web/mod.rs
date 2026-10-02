//! Lowers the host world onto `jco`-transpiled guests: [`Runtime`] is the
//! object every host import is bound to, and its methods only convert values
//! and call into [`crate::host`]; every check lives there.
//!
//! A resource (`document`, `message-subscription`, ...) is its own
//! `#[wasm_bindgen]` struct, returned directly from the call that minted it:
//! jco calls an imported resource's methods straight on the JS value a host
//! import returned, so no dummy instance is ever constructed. jco still
//! validates each one with `instanceof` before that, so `runtime.ts` passes
//! the real classes — read from `window.wasmBindings`, which Trunk's own
//! bootstrap already assigns the glue module's full namespace to — never a
//! throwaway instance reflected for its `.constructor`.
//!
//! Every binding method borrows its script's [`HostHandle`] `RefCell` for the
//! width of its own call, including across that call's internal `await`s.
//! That is sound because nothing else can run concurrently against the same
//! script: wasm32 has one thread, and under JSPI a guest's whole instance
//! suspends while an async import's promise is pending, so no second call
//! into the same [`Runtime`] starts before this one settles.
#![expect(
    clippy::await_holding_refcell_ref,
    reason = "see the module doc: one script's RefCell is never contended"
)]

use std::{
    cell::RefCell,
    rc::Rc,
};

use wasm_bindgen::prelude::*;

use crate::host::ScriptHost;

mod agent;
mod convert;
mod event;
mod input;
mod peer;
mod physics;
mod portal;
mod scene;
mod shading;
mod storage;

/// Lifecycle imports from `runtime.ts`. Declared `catch` so a rejected
/// promise reaches Rust as `Err`, never an uncaught JS exception, which
/// `engine::web` turns into a trap exactly where native's would.
#[wasm_bindgen(module = "/dist/runtime.js")]
unsafe extern "C" {
    #[wasm_bindgen(catch, js_name = "instantiateScript")]
    pub async fn instantiate_script(
        bytes: &[u8],
        hash: &str,
        name: &str,
        runtime: Runtime,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = "scriptInit")]
    pub async fn script_init(instance: &JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = "scriptUpdate")]
    pub async fn script_update(instance: &JsValue, tick: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = "scriptFixedUpdate")]
    pub async fn script_fixed_update(instance: &JsValue, tick: JsValue)
    -> Result<JsValue, JsValue>;
}

/// `wired:script/lifecycle.tick`, lowered for `update`/`fixed-update`.
pub fn wit_tick(tick: crate::engine::Tick) -> JsValue {
    #[derive(serde::Serialize)]
    struct WireTick {
        dt:    f32,
        time:  f64,
        index: u64,
    }
    convert::to_js(&WireTick {
        dt:    tick.dt,
        time:  tick.time,
        index: tick.index,
    })
}

/// Every resource handle shares one script's host state this way, cloning
/// the `Rc` rather than reaching through a table of integer handles: on
/// wasm32 there is one thread, so a `RefCell` borrow can never race, and a
/// lifecycle call never re-enters the same script's [`Runtime`] (the engine
/// will not start a script's next call while one is still in flight).
pub type HostHandle = Rc<RefCell<ScriptHost>>;

/// The object `runtime.ts` binds every host import to.
#[wasm_bindgen]
pub struct Runtime {
    host: HostHandle,
}

impl Runtime {
    #[must_use]
    pub const fn new(host: HostHandle) -> Self {
        Self { host }
    }
}

#[wasm_bindgen]
impl Runtime {
    /// `wired:script/host.granted`.
    #[wasm_bindgen(js_name = "granted")]
    pub fn granted(&self, permission: &str) -> bool {
        convert::permission(permission).is_ok_and(|api| self.host.borrow().granted(api))
    }

    /// A run of the guest's output, gathered by `runtime.ts` over one
    /// microtask, which under JSPI is the guest's synchronous stretch, then
    /// logged the same way native's stdout/stderr runs are.
    ///
    /// Takes `self` to reach it as a method on the `Runtime` `runtime.ts`
    /// already holds; without a receiver `wasm_bindgen` would put it on the
    /// class instead.
    #[expect(clippy::unused_self, reason = "see above")]
    #[wasm_bindgen(js_name = "scriptLog")]
    pub fn script_log(&self, script: &str, is_error: bool, run: &str) {
        use crate::engine::log::{
            Level,
            emit,
        };
        emit(
            script,
            if is_error { Level::Warn } else { Level::Info },
            run,
        );
    }
}

/// Describes a rejected `instantiate`/`init`/`update`/`fixed-update` promise
/// for the log line that retires the script.
pub fn describe_js_error(err: &JsValue) -> String {
    err.as_string().unwrap_or_else(|| format!("{err:?}"))
}
