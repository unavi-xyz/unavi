use bytes::Bytes;
use loro::{LoroMap, LoroValue, ValueOrContainer};
use wasmtime::component::Resource;

use super::bindings::wired::scene::types::{Indices, Mesh, PrimitiveTopology};
use crate::api::wired::scene::WiredSceneRt;

pub struct HostMesh {
    pub index: usize,
}

// Upload f32 values as a blob; store null if vals is None.
async fn write_attr(
    actor: &wds::actor::Actor,
    attrs: &LoroMap,
    key: &str,
    vals: Option<Vec<f32>>,
) -> wasmtime::Result<()> {
    match vals {
        None => attrs
            .insert(key, LoroValue::Null)
            .map_err(|e| anyhow::anyhow!("{e}")),
        Some(v) => {
            let bytes: Vec<u8> = v.iter().flat_map(|f| f.to_le_bytes()).collect();
            let hash = actor
                .upload_blob(Bytes::from(bytes))
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            attrs
                .insert(key, hash.as_bytes().to_vec())
                .map_err(|e| anyhow::anyhow!("{e}"))
        }
    }
}

// Fetch blob by hash; deserialize as f32 slice.
async fn read_attr(
    blobs: &wds::Blobs,
    attrs: &LoroMap,
    key: &str,
) -> wasmtime::Result<Option<Vec<f32>>> {
    let Some(ValueOrContainer::Value(LoroValue::Binary(hash_bytes))) = attrs.get(key) else {
        return Ok(None);
    };
    let arr: [u8; 32] = hash_bytes[..]
        .try_into()
        .map_err(|_| anyhow::anyhow!("invalid hash length"))?;
    let bytes = blobs
        .get_bytes(blake3::Hash::from_bytes(arr))
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let chunks = bytes.chunks_exact(4);
    if !chunks.remainder().is_empty() {
        return Ok(None);
    }
    Ok(Some(
        chunks
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect(),
    ))
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
        let Some(ValueOrContainer::Value(LoroValue::Binary(hash_bytes))) = map.get("indices")
        else {
            return Ok(None);
        };
        let arr: [u8; 32] = hash_bytes[..]
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid hash length"))?;
        let blobs = self
            .blobs
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no wds"))?;
        let bytes = blobs
            .get_bytes(blake3::Hash::from_bytes(arr))
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let values: Vec<u32> = bytes
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        Ok(Some(Indices::Full(values)))
    }

    async fn set_indices(
        &mut self,
        self_: Resource<Mesh>,
        value: Option<Indices>,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        let map = self.mesh_map(index)?;
        let actor = self
            .actor
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no wds"))?;
        match value {
            None => map
                .insert("indices", LoroValue::Null)
                .map_err(|e| anyhow::anyhow!("{e}")),
            Some(Indices::Half(v)) => {
                let bytes: Vec<u8> = v.iter().flat_map(|&x| u32::from(x).to_le_bytes()).collect();
                let hash = actor
                    .upload_blob(Bytes::from(bytes))
                    .await
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                map.insert("indices", hash.as_bytes().to_vec())
                    .map_err(|e| anyhow::anyhow!("{e}"))
            }
            Some(Indices::Full(v)) => {
                let bytes: Vec<u8> = v.iter().flat_map(|i| i.to_le_bytes()).collect();
                let hash = actor
                    .upload_blob(Bytes::from(bytes))
                    .await
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                map.insert("indices", hash.as_bytes().to_vec())
                    .map_err(|e| anyhow::anyhow!("{e}"))
            }
        }
    }

    async fn colors(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Vec<f32>>> {
        let index = self.table.get(&self_)?.index;
        let attrs = self.mesh_attrs(index)?;
        let blobs = self
            .blobs
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no wds"))?;
        read_attr(&blobs, &attrs, "COLOR").await
    }

    async fn set_colors(
        &mut self,
        self_: Resource<Mesh>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        let attrs = self.mesh_attrs(index)?;
        let actor = self
            .actor
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no wds"))?;
        write_attr(&actor, &attrs, "COLOR", values).await
    }

    async fn normals(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Vec<f32>>> {
        let index = self.table.get(&self_)?.index;
        let attrs = self.mesh_attrs(index)?;
        let blobs = self
            .blobs
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no wds"))?;
        read_attr(&blobs, &attrs, "NORMAL").await
    }

    async fn set_normals(
        &mut self,
        self_: Resource<Mesh>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        let attrs = self.mesh_attrs(index)?;
        let actor = self
            .actor
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no wds"))?;
        write_attr(&actor, &attrs, "NORMAL", values).await
    }

    async fn positions(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Vec<f32>>> {
        let index = self.table.get(&self_)?.index;
        let attrs = self.mesh_attrs(index)?;
        let blobs = self
            .blobs
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no wds"))?;
        read_attr(&blobs, &attrs, "POSITION").await
    }

    async fn set_positions(
        &mut self,
        self_: Resource<Mesh>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        let attrs = self.mesh_attrs(index)?;
        let actor = self
            .actor
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no wds"))?;
        write_attr(&actor, &attrs, "POSITION", values).await
    }

    async fn tangents(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Vec<f32>>> {
        let index = self.table.get(&self_)?.index;
        let attrs = self.mesh_attrs(index)?;
        let blobs = self
            .blobs
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no wds"))?;
        read_attr(&blobs, &attrs, "TANGENT").await
    }

    async fn set_tangents(
        &mut self,
        self_: Resource<Mesh>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        let attrs = self.mesh_attrs(index)?;
        let actor = self
            .actor
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no wds"))?;
        write_attr(&actor, &attrs, "TANGENT", values).await
    }

    async fn uv0(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Vec<f32>>> {
        let index = self.table.get(&self_)?.index;
        let attrs = self.mesh_attrs(index)?;
        let blobs = self
            .blobs
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no wds"))?;
        read_attr(&blobs, &attrs, "UV_0").await
    }

    async fn set_uv0(
        &mut self,
        self_: Resource<Mesh>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        let attrs = self.mesh_attrs(index)?;
        let actor = self
            .actor
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no wds"))?;
        write_attr(&actor, &attrs, "UV_0", values).await
    }

    async fn uv1(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Vec<f32>>> {
        let index = self.table.get(&self_)?.index;
        let attrs = self.mesh_attrs(index)?;
        let blobs = self
            .blobs
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no wds"))?;
        read_attr(&blobs, &attrs, "UV_1").await
    }

    async fn set_uv1(
        &mut self,
        self_: Resource<Mesh>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        let index = self.table.get(&self_)?.index;
        let attrs = self.mesh_attrs(index)?;
        let actor = self
            .actor
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no wds"))?;
        write_attr(&actor, &attrs, "UV_1", values).await
    }

    async fn drop(&mut self, rep: Resource<Mesh>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
