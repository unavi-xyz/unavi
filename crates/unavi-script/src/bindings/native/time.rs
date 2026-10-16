//! `wired:core/time`.

use super::{
    HostCtx,
    generated::wired::core::time::Host,
};
use crate::host::time;

impl Host for HostCtx {
    fn utc_offset(&mut self) -> wasmtime::Result<i32> {
        Ok(time::utc_offset())
    }
}
