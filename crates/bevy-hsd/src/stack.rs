//! The layers every document entity stacks above its base. Session layers,
//! one per present peer, are stacked above these by whoever replicates them.

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

/// Stacks override and runtime above the base, weakest first, and points
/// plain writes at runtime. A layer already present is kept.
pub fn install(state: &mut HsdState) -> Result<(), StackError> {
    for (id, merge) in [(OVERRIDE, Merge::Replace), (RUNTIME, Merge::Latest)] {
        match state.add_layer(id, merge, Mask::all()) {
            Ok(()) | Err(StackError::Duplicate(_)) => {}
            Err(err) => return Err(err),
        }
    }
    state.set_edit_target(RUNTIME)
}
