//! This node's store, as resources. The client or server inserts
//! [`DataStore`] once the store is built.

use bevy::prelude::*;
use iroh::{
    EndpointAddr,
    EndpointId,
};
use unavi_store::Store;

/// This node's documents and blobs.
#[derive(Resource, Clone, Deref)]
pub struct DataStore(pub Store);

/// Endpoints this node syncs its documents with.
#[derive(Resource, Default)]
pub struct SyncTargets(pub Vec<EndpointAddr>);

/// Endpoints that hold content beyond the configured sync targets.
///
/// A space's document syncs from its occupants, so its content lives with them
/// too; a fetch offered only the sync targets asks a server that may never
/// have seen the space.
#[derive(Resource, Default)]
pub struct BlobProviders(pub Vec<EndpointId>);
