use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    native::wired::scene::bindings::wired::scene::types::{AlphaMode, Color, HostMaterial},
    shared::{self, wired::scene::material::MaterialRes},
};

impl HostMaterial for Runtime {
    async fn id(&mut self, _self_: Resource<MaterialRes>) -> wasmtime::Result<String> {
        Ok(String::new())
    }

    async fn clone(
        &mut self,
        self_: Resource<MaterialRes>,
    ) -> wasmtime::Result<Resource<MaterialRes>> {
        shared::wired::scene::material::clone(&self.api, self_.rep())
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn name(&mut self, _self_: Resource<MaterialRes>) -> wasmtime::Result<Option<String>> {
        Ok(None)
    }

    async fn set_name(
        &mut self,
        _self_: Resource<MaterialRes>,
        _value: Option<String>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn alpha_cutoff(&mut self, _self_: Resource<MaterialRes>) -> wasmtime::Result<f32> {
        Ok(0.5)
    }

    async fn set_alpha_cutoff(
        &mut self,
        _self_: Resource<MaterialRes>,
        _value: f32,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn alpha_mode(
        &mut self,
        _self_: Resource<MaterialRes>,
    ) -> wasmtime::Result<Option<AlphaMode>> {
        Ok(None)
    }

    async fn set_alpha_mode(
        &mut self,
        _self_: Resource<MaterialRes>,
        _value: Option<AlphaMode>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn base_color(&mut self, _self_: Resource<MaterialRes>) -> wasmtime::Result<Color> {
        Ok(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        })
    }

    async fn set_base_color(
        &mut self,
        _self_: Resource<MaterialRes>,
        _value: Color,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn metallic(&mut self, _self_: Resource<MaterialRes>) -> wasmtime::Result<f32> {
        Ok(0.0)
    }

    async fn set_metallic(
        &mut self,
        _self_: Resource<MaterialRes>,
        _value: f32,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn roughness(&mut self, _self_: Resource<MaterialRes>) -> wasmtime::Result<f32> {
        Ok(0.5)
    }

    async fn set_roughness(
        &mut self,
        _self_: Resource<MaterialRes>,
        _value: f32,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn double_sided(&mut self, _self_: Resource<MaterialRes>) -> wasmtime::Result<bool> {
        Ok(false)
    }

    async fn set_double_sided(
        &mut self,
        _self_: Resource<MaterialRes>,
        _value: bool,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn unlit(&mut self, _self_: Resource<MaterialRes>) -> wasmtime::Result<bool> {
        Ok(false)
    }

    async fn set_unlit(
        &mut self,
        _self_: Resource<MaterialRes>,
        _value: bool,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<MaterialRes>) -> wasmtime::Result<()> {
        shared::wired::scene::material::on_drop(&self.api, rep.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }
}
