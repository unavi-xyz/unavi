use wasm_bindgen::prelude::*;

use crate::{
    engine::log::{
        Level,
        emit,
    },
    runtime::Runtime,
};

#[wasm_bindgen]
impl Runtime {
    /// A run of the guest's output, gathered by `runtime.ts` over one
    /// microtask, which under JSPI is the guest's synchronous stretch.
    ///
    /// The shim it replaces writes straight to the console, so what a script
    /// printed arrived neither batched nor under the client's log filter.
    // Takes `self` to reach `runtime.ts` as a method on the runtime it already
    // holds. Without a receiver `wasm_bindgen` puts it on the class instead.
    #[expect(clippy::unused_self)]
    #[wasm_bindgen(js_name = "scriptLog")]
    pub fn script_log(&self, script: &str, is_error: bool, run: &str) {
        emit(
            script,
            if is_error { Level::Warn } else { Level::Info },
            run,
        );
    }
}
