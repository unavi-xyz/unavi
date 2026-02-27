use loro::{Container, LoroList, LoroValue, ValueOrContainer};
use wasmtime::component::Resource;

use super::bindings::wired::scene::types::Material;
use crate::api::wired::scene::WiredSceneRt;

pub struct HostMaterial {
    pub index: usize,
}

impl super::bindings::wired::scene::types::HostMaterial for WiredSceneRt {
    async fn name(&mut self, self_: Resource<Material>) -> wasmtime::Result<Option<String>> {
        let index = self.table.get(&self_)?.index;
        let map = self.material_map(index)?;
        let name = match map.get("name") {
            Some(ValueOrContainer::Value(LoroValue::String(s))) => Some(s.to_string()),
            _ => None,
        };
        Ok(name)
    }

    async fn set_name(
        &mut self,
        self_: Resource<Material>,
        value: Option<String>,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        let map = self.material_map(index)?;
        value
            .map_or_else(
                || map.insert("name", LoroValue::Null),
                |s| map.insert("name", s.as_str()),
            )
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    async fn base_color(&mut self, self_: Resource<Material>) -> wasmtime::Result<Vec<f32>> {
        let index = self.table.get(&self_)?.index;
        let map = self.material_map(index)?;
        let Some(ValueOrContainer::Container(Container::List(list))) = map.get("base_color") else {
            return Ok(vec![1.0, 1.0, 1.0, 1.0]);
        };
        let LoroValue::List(items) = list.get_deep_value() else {
            return Ok(vec![1.0, 1.0, 1.0, 1.0]);
        };
        #[expect(clippy::cast_possible_truncation)]
        let result = items
            .iter()
            .map(|v| match v {
                LoroValue::Double(d) => *d as f32,
                _ => 0.0,
            })
            .collect();
        Ok(result)
    }

    async fn set_base_color(
        &mut self,
        self_: Resource<Material>,
        value: Vec<f32>,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        let map = self.material_map(index)?;
        let list = map
            .get_or_create_container("base_color", LoroList::new())
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let len = list.len();
        if len > 0 {
            list.delete(0, len).map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        for v in value {
            list.push(LoroValue::Double(f64::from(v)))
                .map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        Ok(())
    }

    async fn metallic(&mut self, self_: Resource<Material>) -> wasmtime::Result<f32> {
        let index = self.table.get(&self_)?.index;
        let map = self.material_map(index)?;
        #[expect(clippy::cast_possible_truncation)]
        let v = match map.get("metallic") {
            Some(ValueOrContainer::Value(LoroValue::Double(d))) => d as f32,
            _ => 0.0,
        };
        Ok(v)
    }

    async fn set_metallic(
        &mut self,
        self_: Resource<Material>,
        value: f32,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        let map = self.material_map(index)?;
        map.insert("metallic", f64::from(value))
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    async fn roughness(&mut self, self_: Resource<Material>) -> wasmtime::Result<f32> {
        let index = self.table.get(&self_)?.index;
        let map = self.material_map(index)?;
        #[expect(clippy::cast_possible_truncation)]
        let v = match map.get("roughness") {
            Some(ValueOrContainer::Value(LoroValue::Double(d))) => d as f32,
            _ => 0.5,
        };
        Ok(v)
    }

    async fn set_roughness(
        &mut self,
        self_: Resource<Material>,
        value: f32,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        let map = self.material_map(index)?;
        map.insert("roughness", f64::from(value))
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    async fn double_sided(&mut self, self_: Resource<Material>) -> wasmtime::Result<bool> {
        let index = self.table.get(&self_)?.index;
        let map = self.material_map(index)?;
        let v = match map.get("double_sided") {
            Some(ValueOrContainer::Value(LoroValue::Bool(b))) => b,
            _ => false,
        };
        Ok(v)
    }

    async fn set_double_sided(
        &mut self,
        self_: Resource<Material>,
        value: bool,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        let map = self.material_map(index)?;
        map.insert("double_sided", value)
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    async fn drop(&mut self, rep: Resource<Material>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
