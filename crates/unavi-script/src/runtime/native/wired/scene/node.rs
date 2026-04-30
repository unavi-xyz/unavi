use crate::runtime::native::{Runtime, wired::scene::bindings::wired::scene::types::HostNode};

pub struct NodeRes;

// impl HostNode for Runtime {
//     async fn id(
//         &mut self,
//         self_: wasmtime::component::Resource<NodeRes>,
//     ) -> wasmtime::Result<String> {
//         todo!()
//     }
//
//     async fn name(
//         &mut self,
//         self_: wasmtime::component::Resource<NodeRes>,
//     ) -> wasmtime::Result<Option<String>> {
//         todo!()
//     }
//
//     async fn drop(&mut self, rep: wasmtime::component::Resource<NodeRes>) -> wasmtime::Result<()> {
//         todo!()
//     }
// }
