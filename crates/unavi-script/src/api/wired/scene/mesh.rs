use loro::{Container, LoroMap, LoroValue, ValueOrContainer};
use wasmtime::component::Resource;

use super::bindings::wired::scene::types::{Indices, Mesh, PrimitiveTopology};
use crate::api::wired::scene::WiredSceneRt;

pub struct HostMesh {
    pub index: usize,
}

// Read a vertex attribute from the "attributes" nested map as a Vec<f32>.
fn read_attr(map: &LoroMap, key: &str) -> Option<Vec<f32>> {
    let ValueOrContainer::Container(Container::Map(attrs)) = map.get("attributes")? else {
        return None;
    };
    let ValueOrContainer::Value(LoroValue::Binary(bytes)) = attrs.get(key)? else {
        return None;
    };
    let chunks = bytes.chunks_exact(4);
    if chunks.remainder().is_empty() {
        Some(
            chunks
                .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect(),
        )
    } else {
        None
    }
}

// Write a vertex attribute to the "attributes" nested map as raw f32 bytes.
fn write_attr(map: &LoroMap, key: &str, vals: Option<Vec<f32>>) -> wasmtime::Result<()> {
    let attrs = map
        .get_or_create_container("attributes", LoroMap::new())
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    vals.map_or_else(
        || {
            attrs
                .insert(key, LoroValue::Null)
                .map_err(|e| anyhow::anyhow!("{e}"))
        },
        |v| {
            let bytes: Vec<u8> = v.iter().flat_map(|f| f.to_le_bytes()).collect();
            attrs
                .insert(key, bytes.as_slice())
                .map_err(|e| anyhow::anyhow!("{e}"))
        },
    )
}

impl super::bindings::wired::scene::types::HostMesh for WiredSceneRt {
    async fn name(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<String>> {
        let index = self.table.get(&self_)?.index;
        let map = self.mesh_map(index)?;
        let name = match map.get("name") {
            Some(ValueOrContainer::Value(LoroValue::String(s))) => Some(s.to_string()),
            _ => None,
        };
        Ok(name)
    }

    async fn set_name(
        &mut self,
        self_: Resource<Mesh>,
        value: Option<String>,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        let map = self.mesh_map(index)?;
        value
            .map_or_else(
                || map.insert("name", LoroValue::Null),
                |s| map.insert("name", s.as_str()),
            )
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    async fn topology(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<PrimitiveTopology> {
        let index = self.table.get(&self_)?.index;
        let map = self.mesh_map(index)?;
        let topo = match map.get("topology") {
            Some(ValueOrContainer::Value(LoroValue::I64(i))) => i,
            _ => 3, // default: TriangleList
        };
        let result = match topo {
            0 => PrimitiveTopology::PointList,
            1 => PrimitiveTopology::LineList,
            2 => PrimitiveTopology::LineStrip,
            4 => PrimitiveTopology::TriangleStrip,
            _ => PrimitiveTopology::TriangleList,
        };
        Ok(result)
    }

    async fn set_topology(
        &mut self,
        self_: Resource<Mesh>,
        value: PrimitiveTopology,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        let map = self.mesh_map(index)?;
        let topo: i64 = match value {
            PrimitiveTopology::PointList => 0,
            PrimitiveTopology::LineList => 1,
            PrimitiveTopology::LineStrip => 2,
            PrimitiveTopology::TriangleList => 3,
            PrimitiveTopology::TriangleStrip => 4,
        };
        map.insert("topology", topo)
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    async fn indices(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Indices>> {
        let index = self.table.get(&self_)?.index;
        let map = self.mesh_map(index)?;
        let Some(ValueOrContainer::Value(LoroValue::Binary(bytes))) = map.get("indices") else {
            return Ok(None);
        };
        let kind = match map.get("indices_type") {
            Some(ValueOrContainer::Value(LoroValue::I64(i))) => i,
            _ => 1, // default u32
        };
        let indices = if kind == 0 {
            let values: Vec<u16> = bytes
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            Indices::Half(values)
        } else {
            let values: Vec<u32> = bytes
                .chunks_exact(4)
                .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            Indices::Full(values)
        };
        Ok(Some(indices))
    }

    async fn set_indices(
        &mut self,
        self_: Resource<Mesh>,
        value: Option<Indices>,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        let map = self.mesh_map(index)?;
        match value {
            None => {
                map.insert("indices", LoroValue::Null)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                map.insert("indices_type", LoroValue::Null)
                    .map_err(|e| anyhow::anyhow!("{e}"))
            }
            Some(Indices::Half(v)) => {
                let bytes: Vec<u8> = v.iter().flat_map(|i| i.to_le_bytes()).collect();
                map.insert("indices", bytes.as_slice())
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                map.insert("indices_type", 0i64)
                    .map_err(|e| anyhow::anyhow!("{e}"))
            }
            Some(Indices::Full(v)) => {
                let bytes: Vec<u8> = v.iter().flat_map(|i| i.to_le_bytes()).collect();
                map.insert("indices", bytes.as_slice())
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                map.insert("indices_type", 1i64)
                    .map_err(|e| anyhow::anyhow!("{e}"))
            }
        }
    }

    async fn colors(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Vec<f32>>> {
        let index = self.table.get(&self_)?.index;
        Ok(read_attr(&self.mesh_map(index)?, "COLOR"))
    }

    async fn set_colors(
        &mut self,
        self_: Resource<Mesh>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        write_attr(&self.mesh_map(index)?, "COLOR", values)
    }

    async fn normals(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Vec<f32>>> {
        let index = self.table.get(&self_)?.index;
        Ok(read_attr(&self.mesh_map(index)?, "NORMAL"))
    }

    async fn set_normals(
        &mut self,
        self_: Resource<Mesh>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        write_attr(&self.mesh_map(index)?, "NORMAL", values)
    }

    async fn positions(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Vec<f32>>> {
        let index = self.table.get(&self_)?.index;
        Ok(read_attr(&self.mesh_map(index)?, "POSITION"))
    }

    async fn set_positions(
        &mut self,
        self_: Resource<Mesh>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        write_attr(&self.mesh_map(index)?, "POSITION", values)
    }

    async fn tangents(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Vec<f32>>> {
        let index = self.table.get(&self_)?.index;
        Ok(read_attr(&self.mesh_map(index)?, "TANGENT"))
    }

    async fn set_tangents(
        &mut self,
        self_: Resource<Mesh>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        write_attr(&self.mesh_map(index)?, "TANGENT", values)
    }

    async fn uv0(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Vec<f32>>> {
        let index = self.table.get(&self_)?.index;
        Ok(read_attr(&self.mesh_map(index)?, "UV_0"))
    }

    async fn set_uv0(
        &mut self,
        self_: Resource<Mesh>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        write_attr(&self.mesh_map(index)?, "UV_0", values)
    }

    async fn uv1(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Vec<f32>>> {
        let index = self.table.get(&self_)?.index;
        Ok(read_attr(&self.mesh_map(index)?, "UV_1"))
    }

    async fn set_uv1(
        &mut self,
        self_: Resource<Mesh>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        write_attr(&self.mesh_map(index)?, "UV_1", values)
    }

    async fn drop(&mut self, rep: Resource<Mesh>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
