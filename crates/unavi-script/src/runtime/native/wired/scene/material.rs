use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    native::wired::scene::bindings::wired::scene::types::{AlphaMode, Color, HostMaterial},
    shared::{
        self,
        wired::scene::material::{MaterialAlphaMode, MaterialColor, MaterialRes},
    },
};

const fn wit_to_alpha_mode(m: AlphaMode) -> MaterialAlphaMode {
    match m {
        AlphaMode::Add => MaterialAlphaMode::Add,
        AlphaMode::Blend => MaterialAlphaMode::Blend,
        AlphaMode::Mask => MaterialAlphaMode::Mask,
        AlphaMode::Multiply => MaterialAlphaMode::Multiply,
        AlphaMode::Opaque => MaterialAlphaMode::Opaque,
        AlphaMode::PreMultiplied => MaterialAlphaMode::PreMultiplied,
    }
}

const fn alpha_mode_to_wit(m: MaterialAlphaMode) -> AlphaMode {
    match m {
        MaterialAlphaMode::Add => AlphaMode::Add,
        MaterialAlphaMode::Blend => AlphaMode::Blend,
        MaterialAlphaMode::Mask => AlphaMode::Mask,
        MaterialAlphaMode::Multiply => AlphaMode::Multiply,
        MaterialAlphaMode::Opaque => AlphaMode::Opaque,
        MaterialAlphaMode::PreMultiplied => AlphaMode::PreMultiplied,
    }
}

const fn wit_to_color(c: Color) -> MaterialColor {
    MaterialColor {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    }
}

const fn color_to_wit(c: MaterialColor) -> Color {
    Color {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    }
}

impl HostMaterial for Runtime {
    async fn id(&mut self, self_: Resource<MaterialRes>) -> wasmtime::Result<String> {
        shared::wired::scene::material::id(&self.api, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn clone(
        &mut self,
        self_: Resource<MaterialRes>,
    ) -> wasmtime::Result<Resource<MaterialRes>> {
        shared::wired::scene::material::clone(&self.api, self_.rep())
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn name(&mut self, self_: Resource<MaterialRes>) -> wasmtime::Result<Option<String>> {
        shared::wired::scene::material::name(&self.api, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_name(
        &mut self,
        self_: Resource<MaterialRes>,
        value: Option<String>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::material::set_name(&self.api, self_.rep(), value)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn alpha_cutoff(&mut self, self_: Resource<MaterialRes>) -> wasmtime::Result<f32> {
        shared::wired::scene::material::alpha_cutoff(&self.api, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_alpha_cutoff(
        &mut self,
        self_: Resource<MaterialRes>,
        value: f32,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::material::set_alpha_cutoff(&self.api, self_.rep(), value)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn alpha_mode(
        &mut self,
        self_: Resource<MaterialRes>,
    ) -> wasmtime::Result<Option<AlphaMode>> {
        shared::wired::scene::material::alpha_mode(&self.api, self_.rep())
            .map(|opt| opt.map(alpha_mode_to_wit))
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_alpha_mode(
        &mut self,
        self_: Resource<MaterialRes>,
        value: Option<AlphaMode>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::material::set_alpha_mode(
            &self.api,
            self_.rep(),
            value.map(wit_to_alpha_mode),
        )
        .map_err(wasmtime::Error::from_anyhow)
    }

    async fn base_color(&mut self, self_: Resource<MaterialRes>) -> wasmtime::Result<Color> {
        shared::wired::scene::material::base_color(&self.api, self_.rep())
            .map(color_to_wit)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_base_color(
        &mut self,
        self_: Resource<MaterialRes>,
        value: Color,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::material::set_base_color(&self.api, self_.rep(), wit_to_color(value))
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn metallic(&mut self, self_: Resource<MaterialRes>) -> wasmtime::Result<f32> {
        shared::wired::scene::material::metallic(&self.api, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_metallic(
        &mut self,
        self_: Resource<MaterialRes>,
        value: f32,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::material::set_metallic(&self.api, self_.rep(), value)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn roughness(&mut self, self_: Resource<MaterialRes>) -> wasmtime::Result<f32> {
        shared::wired::scene::material::roughness(&self.api, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_roughness(
        &mut self,
        self_: Resource<MaterialRes>,
        value: f32,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::material::set_roughness(&self.api, self_.rep(), value)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn double_sided(&mut self, self_: Resource<MaterialRes>) -> wasmtime::Result<bool> {
        shared::wired::scene::material::double_sided(&self.api, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_double_sided(
        &mut self,
        self_: Resource<MaterialRes>,
        value: bool,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::material::set_double_sided(&self.api, self_.rep(), value)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn unlit(&mut self, self_: Resource<MaterialRes>) -> wasmtime::Result<bool> {
        shared::wired::scene::material::unlit(&self.api, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_unlit(
        &mut self,
        self_: Resource<MaterialRes>,
        value: bool,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::material::set_unlit(&self.api, self_.rep(), value)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn drop(&mut self, rep: Resource<MaterialRes>) -> wasmtime::Result<()> {
        shared::wired::scene::material::on_drop(&self.api, rep.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }
}
