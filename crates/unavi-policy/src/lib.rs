//! Access control for peers and documents.
//!
//! # Types
//!
//! | Type | States |
//! | --- | --- |
//! | [`trust::Trust`] | how far a peer is trusted |
//! | [`trust::Threshold`] | the trust a document requires of its writers |
//! | [`permissions::Permissions`] | the host APIs a document may call |
//! | [`owner::Owner`] | who holds a document |
//! | [`standing::Standing`] | the four above, for one document |
//!
//! # Registry
//!
//! [`registry::Policy`] holds one [`registry::Record`] per document id and the
//! quota each document spends against. An unregistered document reads as
//! [`registry::Record::default`].
//!
//! # Checks
//!
//! [`standing::Standing::may_write`] and [`standing::Standing::may_read`] take
//! two `Standing` values and answer with [`error::PolicyError`].
//!
//! # External Inputs
//!
//! [`owner::Owner::Peer`] comes from pin state held outside this crate. The
//! caller resolves it and builds the `Standing`.

use bevy::prelude::*;
use bevy_hsd::{
    HsdCommitSet,
    HsdDocId,
};

use crate::{
    owner::Owner,
    permissions::Permissions,
    trust::Threshold,
};

pub mod error;
pub mod owner;
pub mod permissions;
pub mod quota;
pub mod registry;
pub mod space;
pub mod standing;
pub mod sync;
pub mod trust;

pub struct PolicyPlugin;

impl Plugin for PolicyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<registry::Policy>()
            .add_observer(sync::sync_on::<HsdDocId>)
            .add_observer(sync::sync_on::<Permissions>)
            .add_observer(sync::sync_on::<Threshold>)
            .add_observer(sync::sync_on::<Owner>)
            .add_observer(sync::forget_document)
            .add_observer(space::register_space)
            .add_observer(space::register_membership)
            .add_observer(space::forget_membership)
            .add_observer(space::forget_space)
            .add_systems(Update, space::parent_docs_under_space.before(HsdCommitSet));
    }
}
