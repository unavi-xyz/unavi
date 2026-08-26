use bevy::prelude::*;
use bevy_hsd::HsdCommitSet;

pub mod check;
pub mod document;
pub mod error;
pub mod limits;
pub mod membership;
pub mod reach;
pub mod registry;
pub mod space;
pub mod sync;
pub mod tier;
pub mod trust;

/// Registers the document-policy lifecycle: who a document may talk to, and
/// what it is allowed to call.
///
/// Both `ScriptPlugin` and `SpacePlugin` add this if it is absent, since
/// either can be the root of an app.
pub struct PolicyPlugin;

impl Plugin for PolicyPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(sync::sync_on_doc_id)
            .add_observer(sync::sync_on_policy)
            .add_observer(sync::sync_on_reach)
            .add_observer(sync::forget_document)
            .add_observer(space::grant_space_permissions)
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
