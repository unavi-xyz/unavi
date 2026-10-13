//! The layers every document entity stacks above its base.

use hsd::state::{
    HsdState,
    stack::{
        LayerId,
        Mask,
        Merge,
        StackError,
    },
};

/// The referencing document's opinions about this one.
pub const OVERRIDE: LayerId = LayerId::from_static("override");
/// What this peer's scripts write locally. Plain writes land here.
pub const RUNTIME: LayerId = LayerId::from_static("runtime");
/// Live opinions present peers state.
pub const SESSION: LayerId = LayerId::from_static("session");

/// Stacks override, runtime and session above the base, weakest first, and
/// points plain writes at runtime. A layer already present is kept.
pub fn install(state: &mut HsdState) -> Result<(), StackError> {
    for (id, merge) in [
        (OVERRIDE, Merge::Replace),
        (RUNTIME, Merge::Latest),
        (SESSION, Merge::Latest),
    ] {
        match state.add_layer(id, merge, Mask::all()) {
            Ok(()) | Err(StackError::Duplicate(_)) => {}
            Err(err) => return Err(err),
        }
    }
    state.set_edit_target(RUNTIME)
}
