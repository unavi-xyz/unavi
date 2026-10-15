//! Bevy integration for iroh: the endpoint, its router, this node's store as
//! resources, blob fetches, and the `iroh://` asset source.

use bevy::prelude::*;

pub mod assets;
pub mod blob;
pub mod endpoint;
pub mod router;
pub mod store;

/// Receives results from async iroh work. Runs in `PreUpdate`, so a result is
/// visible to the frame's `Update` systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct IrohSystems;

/// Binds endpoints on [`endpoint::LoadEndpoint`], builds routers on
/// [`router::BuildRouter`], and fetches blobs for [`blob::BlobRequest`]s.
pub struct IrohPlugin;

impl Plugin for IrohPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<store::SyncTargets>()
            .init_resource::<store::BlobProviders>()
            .add_observer(endpoint::on_load_endpoint)
            .add_observer(router::on_build_router)
            .add_observer(blob::on_blob_request_insert)
            .add_observer(blob::on_blob_request_remove)
            .add_systems(
                PreUpdate,
                (
                    endpoint::receive_endpoint,
                    router::receive_router,
                    blob::recv_blob_responses,
                )
                    .in_set(IrohSystems),
            );
    }
}
