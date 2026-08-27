//! What a document may reach, what it may call, and what it may consume.
//!
//! Every question here is answered from two facts: the rung the local user puts
//! a *peer* at ([`trust::Trust`]), and where a *document* was loaded from
//! ([`tier::Tier`]). Reach between documents is derived rather than stored.
//!
//! The predicates are pure and the registry is a value. Resolving who owns a
//! document needs the network state, which lives above this crate, so the
//! composed checks live there too.

use bevy::prelude::*;
use bevy_hsd::HsdCommitSet;

pub mod document;
pub mod error;
pub mod membership;
pub mod quota;
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
        app.init_resource::<registry::Policy>()
            .add_observer(sync::sync_on_doc_id)
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
