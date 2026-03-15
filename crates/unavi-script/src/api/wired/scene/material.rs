use std::sync::{Arc, atomic::Ordering};

use bevy_hsd::cache::MaterialInner;
use wasmtime::component::Resource;

use super::bindings::wired::scene::types::Material;
use crate::api::wired::scene::WiredSceneRt;

#[derive(Clone)]
pub struct HostMaterial {
    pub inner: Arc<MaterialInner>,
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

    async fn commit(&mut self, self_: Resource<Material>) -> wasmtime::Result<()> {
        self.check_hsd_write()?;
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let state = inner.state.lock().expect("material state lock");
        let map = self.material_map(&inner.id)?;

        let [r, g, b, a] = state.base_color;
        let base_color_list = map
            .get_or_create_container("base_color", loro::LoroList::new())
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let len = base_color_list.len();
        if len > 0 {
            base_color_list
                .delete(0, len)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        for v in [r, g, b, a] {
            base_color_list
                .push(loro::LoroValue::Double(f64::from(v)))
                .map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        map.insert("metallic", f64::from(state.metallic))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        map.insert("roughness", f64::from(state.roughness))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        map.insert("double_sided", state.double_sided)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        if let Some(name) = &state.name {
            map.insert("name", name.as_str())
                .map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        drop(state);
        self.doc.commit();
        self.doc.compact_change_store();
        self.doc.free_history_cache();
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
        inner.state.lock().expect("material state lock").name = value;
        inner.dirty.store(true, Ordering::Relaxed);
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
        {
            let mut state = inner.state.lock().expect("material state lock");
            let r = value.first().copied().unwrap_or(1.0);
            let g = value.get(1).copied().unwrap_or(1.0);
            let b = value.get(2).copied().unwrap_or(1.0);
            let a = value.get(3).copied().unwrap_or(1.0);
            state.base_color = [r, g, b, a];
        }
        inner.dirty.store(true, Ordering::Relaxed);
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
        inner.dirty.store(true, Ordering::Relaxed);
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
        inner.dirty.store(true, Ordering::Relaxed);
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
        inner.dirty.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<Material>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
