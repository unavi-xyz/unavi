//! Access control for peers and documents.
//!
//! # Types
//!
//! | Type | States |
//! | --- | --- |
//! | [`trust::Trust`] | how far a peer is trusted |
//! | [`permissions::Permissions`] | the host APIs a document may call, derived from its author's trust |
//!
//! # Registry
//!
//! [`registry::Policy`] holds one [`registry::Record`] per document id and the
//! quota each document spends against. An unregistered document reads as
//! [`registry::Record::default`].
//!
//! # Checks
//!
//! [`permissions::Permissions::for_trust`] is the whole grant, and
//! [`permissions::Permissions::require`] the gate, answering with
//! [`error::PolicyError`]. Which peer authored a document is not decided here:
//! authorship is replicated state, resolved by the caller.

use bevy::prelude::*;
use bevy_hsd::HsdSystems;

pub mod error;
pub mod permissions;
pub mod quota;
pub mod registry;
pub mod space;
pub mod sync;
pub mod trust;

pub struct PolicyPlugin;

impl Plugin for PolicyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<registry::Policy>()
            .add_observer(sync::register_document)
            .add_observer(sync::forget_document)
            .add_observer(space::register_space)
            .add_observer(space::register_membership)
            .add_observer(space::forget_membership)
            .add_observer(space::forget_space)
            .add_systems(Update, space::parent_docs_under_space.before(HsdSystems));
    }
}
