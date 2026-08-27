use bevy::prelude::*;
use iroh::{
    EndpointAddr,
    EndpointId,
};
use iroh_blobs::api::{
    blobs::Blobs,
    downloader::Downloader,
};
use unavi_store::store::Store;

/// This node's data plane: documents, blobs, the author it writes under, and
/// its root document.
///
/// The blob-only components below stay separate: the fetch path and the asset
/// loader run without a document store.
#[derive(Component)]
pub struct LocalStore(pub Store);

#[derive(Component)]
#[require(SyncTargets, BlobProviders)]
pub struct LocalBlobs(pub Blobs);

/// The store backing [`LocalBlobs`], for tag management the blobs client does
/// not expose, such as pinning content against garbage collection.
#[derive(Component)]
pub struct LocalBlobStore(pub iroh_blobs::api::Store);

/// Pulls blobs from named providers. Holds internal state, so it is built once
/// with the store rather than per fetch.
#[derive(Component)]
pub struct LocalDownloader(pub Downloader);

/// Endpoints this node syncs its documents with.
#[derive(Component, Default)]
pub struct SyncTargets(pub Vec<EndpointAddr>);

/// Endpoints that hold content beyond the configured sync targets.
///
/// A space's document syncs from its occupants, so its content lives with them
/// too; a fetch offered only the sync targets asks a server that may never
/// have seen the space.
#[derive(Component, Default)]
pub struct BlobProviders(pub Vec<EndpointId>);
