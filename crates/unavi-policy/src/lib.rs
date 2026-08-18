use bevy::prelude::*;
use bevy_hsd::HsdCommitSet;

pub mod document;
pub mod error;
pub mod identity;
pub mod membership;
pub mod reach;
pub mod space;
pub mod tier;
pub mod trust;

/// Registers the document-policy lifecycle: who a document may talk to, and
/// what it is allowed to call.
///
/// Both `ScriptPlugin` and `SpacePlugin` add this if it is absent, since either
/// can be the root of an app — the script examples run without a space.
pub struct PolicyPlugin;

impl Plugin for PolicyPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(reach::register_reach)
            .add_observer(reach::deregister_reach)
            .add_observer(document::grant_space_permissions)
            .add_observer(document::inherit_host_permissions)
            .add_observer(membership::self_own_space)
            .add_observer(membership::register_on_owner_change)
            .add_observer(membership::deregister_doc_membership)
            .add_observer(membership::deregister_space_docs)
            .add_systems(
                Update,
                membership::parent_docs_under_space.before(HsdCommitSet),
            );
    }
}
