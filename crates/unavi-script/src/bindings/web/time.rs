//! `wired:core/time`.

use wasm_bindgen::prelude::*;

use super::Runtime;
use crate::host::time;

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "utcOffset")]
    pub fn utc_offset(&self) -> i32 {
        time::utc_offset()
    }
}
