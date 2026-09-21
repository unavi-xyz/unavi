//! `unavi-policy` determines the capability and access levels of everything in
//! the scene.
//!
//! Peers are restricted by their [`trust::Trust`]. Documents are restricted by
//! the [`threshold::Threshold`] their writers must clear and the
//! [`owner::Owner`] the view resolves for them.

use bevy::prelude::*;
use bevy_hsd::HsdCommitSet;

pub mod document;
pub mod error;
pub mod membership;
pub mod owner;
pub mod quota;
pub mod registry;
pub mod space;
pub mod standing;
pub mod sync;
pub mod threshold;
pub mod trust;

pub struct PolicyPlugin;

impl Plugin for PolicyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<registry::Policy>()
            .add_observer(sync::sync_on_doc_id)
            .add_observer(sync::sync_on_policy)
            .add_observer(sync::sync_on_threshold)
            .add_observer(sync::sync_on_owner)
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
