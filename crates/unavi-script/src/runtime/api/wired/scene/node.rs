use bevy_hsd::cache::NodeInner;

use crate::runtime::api::wired::scene::{WiredSceneRt, bindings::wired::scene::types::HostNode};

pub struct NodeRes {
    inner: NodeInner,
}

// impl HostNode for WiredSceneRt {
//     async fn id(
//         &mut self,
//         self_: wasmtime::component::Resource<NodeRes>,
//     ) -> wasmtime::Result<String> {
//         let id = self.table.get(&self_)?.inner.id.to_string();
//         Ok(id)
//     }
//
//     async fn name(
//         &mut self,
//         self_: wasmtime::component::Resource<NodeRes>,
//     ) -> wasmtime::Result<Option<String>> {
//         todo!()
//     }
// }
