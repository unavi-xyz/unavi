use bevy::prelude::*;
use iroh::{
    EndpointAddr,
    EndpointId,
};
use iroh_blobs::api::{
    blobs::Blobs,
    downloader::Downloader,
};
use iroh_docs::protocol::Docs;
use iroh_gossip::Gossip;

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

#[derive(Component)]
pub struct LocalDocs(pub Docs);

/// The store's gossip, which is the only one on this endpoint: the router
/// accepts `iroh_gossip::ALPN` once, so a second instance would silently take
/// every inbound connection from the first.
#[derive(Component)]
pub struct LocalGossip(pub Gossip);

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
