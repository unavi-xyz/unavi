//! Pipeline from raw Loro diffs to compiled Bevy assets.
//!
//! Flow: `init` → Loro subscription → `diff` → `events::RawChangeQueue`
//! → `queue::process_hsd_queue` → typed observers → `compile`.
//! The `sync` module handles the reverse path (ECS → CRDT write-back).

pub mod compile;
mod diff;
pub mod events;
pub mod init;
pub mod queue;
pub mod sync;
