use std::sync::{Arc, atomic::Ordering};

use bevy_hsd::cache::MaterialInner;
use wasmtime::bail;
use wasmtime::component::Resource;

use super::bindings::wired::scene::types::{AlphaMode, Color, Material};
use crate::api::wired::scene::WiredSceneRt;
use crate::core_ops;

pub struct HostMaterial {
    pub inner: Arc<MaterialInner>,
    pub can_read: bool,
    pub can_write: bool,
}

impl Clone for HostMaterial {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            can_read: self.can_read,
            can_write: self.can_write,
        }
    }
}

impl super::bindings::wired::scene::types::HostMaterial for WiredSceneRt {
    async fn id(
        &mut self,
        self_: wasmtime::component::Resource<HostMaterial>,
    ) -> wasmtime::Result<String> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner.id.to_string())
    }
    async fn clone(
        &mut self,
        self_: wasmtime::component::Resource<HostMaterial>,
    ) -> wasmtime::Result<wasmtime::component::Resource<HostMaterial>> {
        let inner = self.table.get(&self_)?.clone();
        let mat = self.table.push(inner)?;
        Ok(mat)
    }

    async fn sync(&mut self, self_: Resource<Material>) -> wasmtime::Result<bool> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner.sync.load(Ordering::Relaxed))
    }

    async fn set_sync(&mut self, self_: Resource<Material>, value: bool) -> wasmtime::Result<()> {
        let (inner, can_write) = {
            let m = self.table.get(&self_)?;
            (Arc::clone(&m.inner), m.can_write)
        };
        if value && !can_write {
            bail!("hsd write permission required")
        }
        inner.sync.store(value, Ordering::Relaxed);
        Ok(())
    }

    async fn name(&mut self, self_: Resource<Material>) -> wasmtime::Result<Option<String>> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner
            .state
            .lock()
            .expect("material state lock")
            .name
            .clone())
    }

    async fn set_name(
        &mut self,
        self_: Resource<Material>,
        value: Option<String>,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::material::set_name(&inner, self.doc_entity, value, &mut queue);
        Ok(())
    }

    async fn alpha_cutoff(&mut self, self_: Resource<Material>) -> wasmtime::Result<f32> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner
            .state
            .lock()
            .expect("material state lock")
            .alpha_cutoff
            .unwrap_or(0.5))
    }

    async fn set_alpha_cutoff(
        &mut self,
        self_: Resource<Material>,
        value: f32,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::material::set_alpha_cutoff(&inner, self.doc_entity, value, &mut queue);
        Ok(())
    }

    async fn alpha_mode(
        &mut self,
        self_: Resource<Material>,
    ) -> wasmtime::Result<Option<AlphaMode>> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let mode = inner
            .state
            .lock()
            .expect("material state lock")
            .alpha_mode
            .as_deref()
            .and_then(|s| match s {
                "add" => Some(AlphaMode::Add),
                "blend" => Some(AlphaMode::Blend),
                "mask" => Some(AlphaMode::Mask),
                "multiply" => Some(AlphaMode::Multiply),
                "opaque" => Some(AlphaMode::Opaque),
                "premultiplied" => Some(AlphaMode::PreMultiplied),
                _ => None,
            });
        Ok(mode)
    }

    async fn set_alpha_mode(
        &mut self,
        self_: Resource<Material>,
        value: Option<AlphaMode>,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let mode_str = value.map(|m| match m {
            AlphaMode::Add => "add".to_string(),
            AlphaMode::Blend => "blend".to_string(),
            AlphaMode::Mask => "mask".to_string(),
            AlphaMode::Multiply => "multiply".to_string(),
            AlphaMode::Opaque => "opaque".to_string(),
            AlphaMode::PreMultiplied => "premultiplied".to_string(),
        });
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::material::set_alpha_mode(&inner, self.doc_entity, mode_str, &mut queue);
        Ok(())
    }

    async fn base_color(&mut self, self_: Resource<Material>) -> wasmtime::Result<Color> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let [r, g, b, a] = inner.state.lock().expect("material state lock").base_color;
        Ok(Color { r, g, b, a })
    }

    async fn set_base_color(
        &mut self,
        self_: Resource<Material>,
        value: Color,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::material::set_base_color(
            &inner,
            self.doc_entity,
            [value.r, value.g, value.b, value.a],
            &mut queue,
        );
        Ok(())
    }

    async fn metallic(&mut self, self_: Resource<Material>) -> wasmtime::Result<f32> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner.state.lock().expect("material state lock").metallic)
    }

    async fn set_metallic(
        &mut self,
        self_: Resource<Material>,
        value: f32,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::material::set_metallic(&inner, self.doc_entity, value, &mut queue);
        Ok(())
    }

    async fn roughness(&mut self, self_: Resource<Material>) -> wasmtime::Result<f32> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner.state.lock().expect("material state lock").roughness)
    }

    async fn set_roughness(
        &mut self,
        self_: Resource<Material>,
        value: f32,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::material::set_roughness(&inner, self.doc_entity, value, &mut queue);
        Ok(())
    }

    async fn double_sided(&mut self, self_: Resource<Material>) -> wasmtime::Result<bool> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner
            .state
            .lock()
            .expect("material state lock")
            .double_sided)
    }

    async fn set_double_sided(
        &mut self,
        self_: Resource<Material>,
        value: bool,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::material::set_double_sided(&inner, self.doc_entity, value, &mut queue);
        Ok(())
    }

    async fn unlit(&mut self, self_: Resource<Material>) -> wasmtime::Result<bool> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner.state.lock().expect("material state lock").unlit)
    }

    async fn set_unlit(&mut self, self_: Resource<Material>, value: bool) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::material::set_unlit(&inner, self.doc_entity, value, &mut queue);
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<Material>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
