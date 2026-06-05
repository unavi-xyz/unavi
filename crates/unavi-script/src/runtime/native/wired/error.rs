use crate::error::ScriptError;

/// Canonical generation of `wired:error/types`. Every other binding `with`-maps
/// its `wired:error/types` onto this module, so the whole runtime lowers
/// [`ScriptError`] through a single `Error` type.
pub mod bindings {
    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-error",
        imports: { default: async | trappable },
    });
}

pub use bindings::wired::error::types::{
    self,
    Error,
};

impl From<ScriptError> for Error {
    fn from(err: ScriptError) -> Self {
        match err {
            ScriptError::Other(s) => Self::Other(s),
            ScriptError::Quota(_) => Self::Quota,
            ScriptError::Permission(_) => Self::Permission,
            ScriptError::Firewall(_) => Self::Firewall,
        }
    }
}
