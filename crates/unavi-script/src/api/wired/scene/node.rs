use smol_str::SmolStr;

use crate::api::wired::scene::WiredSceneRt;

pub struct HostNode {
    id: SmolStr,
}

impl super::bindings::wired::scene::types::HostNode for WiredSceneRt {
    async fn new(&mut self) -> wasmtime::Result<wasmtime::component::Resource<HostNode>> {
        let id = "todo".into();
        let res = self.table.push(HostNode { id })?;
        Ok(res)
    }

    async fn document(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
    ) -> wasmtime::Result<wasmtime::component::__internal::Vec<u8>> {
        todo!()
    }

    async fn id(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
    ) -> wasmtime::Result<String> {
        let node = self.table.get(&self_)?;
        Ok(node.id.to_string())
    }

    async fn mesh(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
    ) -> wasmtime::Result<
        Option<wasmtime::component::Resource<super::bindings::wired::scene::types::Mesh>>,
    > {
        todo!()
    }
    async fn set_mesh(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
        value: Option<wasmtime::component::Resource<super::bindings::wired::scene::types::Mesh>>,
    ) -> wasmtime::Result<()> {
        todo!()
    }

    async fn drop(&mut self, rep: wasmtime::component::Resource<HostNode>) -> wasmtime::Result<()> {
        todo!()
    }
}
