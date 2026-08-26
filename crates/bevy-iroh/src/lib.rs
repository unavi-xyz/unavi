use bevy::prelude::*;

pub mod blob;
pub mod doc;
pub mod endpoint;
pub mod router;
pub mod store;

pub struct IrohPlugin;

impl Plugin for IrohPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(endpoint::on_load_endpoint)
            .add_observer(router::on_build_router)
            .add_observer(blob::get::on_get_blob)
            .add_observer(blob::request::on_blob_request_add)
            .add_observer(blob::request::on_blob_request_remove)
            .add_observer(doc::on_doc_set)
            .add_observer(doc::on_doc_get)
            .add_observer(doc::on_doc_list)
            .add_systems(
                FixedUpdate,
                (
                    endpoint::receive_endpoint,
                    router::receive_router,
                    blob::deps::mark_blob_deps_loaded,
                    blob::request::recv_blob_responses,
                ),
            );
    }
}
