use std::sync::{Arc, atomic::Ordering};

use bevy_hsd::cache::MaterialInner;
use wasmtime::component::Resource;

use super::bindings::wired::scene::types::{AlphaMode, Material};
use crate::api::wired::scene::WiredSceneRt;

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
            return Err(anyhow::anyhow!("hsd write permission required"));
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
        inner
            .state
            .lock()
            .expect("material state lock")
            .name
            .clone_from(&value);
        if inner.sync.load(Ordering::Relaxed) {
            inner.hsd_changes.lock().expect("hsd_changes lock").name = Some(value);
        } else {
            inner.dirty.lock().expect("dirty lock").name = true;
        }
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
        inner
            .state
            .lock()
            .expect("material state lock")
            .alpha_cutoff = Some(value);
        if inner.sync.load(Ordering::Relaxed) {
            inner
                .hsd_changes
                .lock()
                .expect("hsd_changes lock")
                .alpha_cutoff = Some(f64::from(value));
        } else {
            inner.dirty.lock().expect("dirty lock").alpha_cutoff = true;
        }
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
        inner
            .state
            .lock()
            .expect("material state lock")
            .alpha_mode
            .clone_from(&mode_str);
        if inner.sync.load(Ordering::Relaxed) {
            inner
                .hsd_changes
                .lock()
                .expect("hsd_changes lock")
                .alpha_mode = Some(mode_str);
        } else {
            inner.dirty.lock().expect("dirty lock").alpha_mode = true;
        }
        Ok(())
    }

    async fn base_color(&mut self, self_: Resource<Material>) -> wasmtime::Result<Vec<f32>> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let state = inner.state.lock().expect("material state lock");
        Ok(state.base_color.to_vec())
    }

    async fn set_base_color(
        &mut self,
        self_: Resource<Material>,
        value: Vec<f32>,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let r = value.first().copied().unwrap_or(1.0);
        let g = value.get(1).copied().unwrap_or(1.0);
        let b = value.get(2).copied().unwrap_or(1.0);
        let a = value.get(3).copied().unwrap_or(1.0);
        inner.state.lock().expect("material state lock").base_color = [r, g, b, a];
        if inner.sync.load(Ordering::Relaxed) {
            inner
                .hsd_changes
                .lock()
                .expect("hsd_changes lock")
                .base_color = Some([f64::from(r), f64::from(g), f64::from(b), f64::from(a)]);
        } else {
            inner.dirty.lock().expect("dirty lock").base_color = true;
        }
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
        inner.state.lock().expect("material state lock").metallic = value;
        if inner.sync.load(Ordering::Relaxed) {
            inner.hsd_changes.lock().expect("hsd_changes lock").metallic = Some(f64::from(value));
        } else {
            inner.dirty.lock().expect("dirty lock").metallic = true;
        }
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
        inner.state.lock().expect("material state lock").roughness = value;
        if inner.sync.load(Ordering::Relaxed) {
            inner
                .hsd_changes
                .lock()
                .expect("hsd_changes lock")
                .roughness = Some(f64::from(value));
        } else {
            inner.dirty.lock().expect("dirty lock").roughness = true;
        }
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
        inner
            .state
            .lock()
            .expect("material state lock")
            .double_sided = value;
        if inner.sync.load(Ordering::Relaxed) {
            inner
                .hsd_changes
                .lock()
                .expect("hsd_changes lock")
                .double_sided = Some(value);
        } else {
            inner.dirty.lock().expect("dirty lock").double_sided = true;
        }
        Ok(())
    }

    async fn unlit(&mut self, self_: Resource<Material>) -> wasmtime::Result<bool> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner.state.lock().expect("material state lock").unlit)
    }

    async fn set_unlit(&mut self, self_: Resource<Material>, value: bool) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        inner.state.lock().expect("material state lock").unlit = value;
        if inner.sync.load(Ordering::Relaxed) {
            inner.hsd_changes.lock().expect("hsd_changes lock").unlit = Some(value);
        } else {
            inner.dirty.lock().expect("dirty lock").unlit = true;
        }
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<Material>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
