use bevy::prelude::Transform as BevyTransform;
use loro::{Container, LoroList, LoroMap, LoroValue, TreeParentId, ValueOrContainer};
use wasmtime::component::Resource;

use super::bindings::wired::scene::types::{
    Collider, ColliderCapsule, Material, Mesh, Quat, RigidBodyKind, Transform, Vec3,
};
use crate::api::wired::scene::{WiredSceneRt, material::HostMaterial, mesh::HostMesh};

pub struct HostNode {
    pub tree_id: loro::TreeID,
}

// Read an N-element f64 list from a meta map key.
fn read_f64s<const N: usize>(meta: &LoroMap, key: &str) -> Option<[f64; N]> {
    let ValueOrContainer::Container(Container::List(list)) = meta.get(key)? else {
        return None;
    };
    let LoroValue::List(items) = list.get_deep_value() else {
        return None;
    };
    if items.len() != N {
        return None;
    }
    let mut out = [0f64; N];
    for (i, v) in items.iter().enumerate() {
        let LoroValue::Double(d) = v else { return None };
        out[i] = *d;
    }
    Some(out)
}

// Write an f64 slice to a meta map key (delete-then-push).
fn write_f64s(meta: &LoroMap, key: &str, vals: &[f64]) -> wasmtime::Result<()> {
    let list = meta
        .get_or_create_container(key, LoroList::new())
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let len = list.len();
    if len > 0 {
        list.delete(0, len).map_err(|e| anyhow::anyhow!("{e}"))?;
    }
    for &v in vals {
        list.push(LoroValue::Double(v))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
    }
    Ok(())
}

// Build a bevy Transform from a meta map.
fn bevy_transform(meta: &LoroMap) -> BevyTransform {
    let t = read_f64s::<3>(meta, "translation").unwrap_or([0.0, 0.0, 0.0]);
    let r = read_f64s::<4>(meta, "rotation").unwrap_or([0.0, 0.0, 0.0, 1.0]);
    let s = read_f64s::<3>(meta, "scale").unwrap_or([1.0, 1.0, 1.0]);
    #[expect(clippy::cast_possible_truncation)]
    BevyTransform {
        translation: bevy::math::Vec3::new(t[0] as f32, t[1] as f32, t[2] as f32),
        rotation: bevy::math::Quat::from_xyzw(r[0] as f32, r[1] as f32, r[2] as f32, r[3] as f32),
        scale: bevy::math::Vec3::new(s[0] as f32, s[1] as f32, s[2] as f32),
    }
}

impl super::bindings::wired::scene::types::HostNode for WiredSceneRt {
    async fn name(&mut self, self_: Resource<HostNode>) -> wasmtime::Result<Option<String>> {
        let id = self.table.get(&self_)?.tree_id;
        let meta = self.node_meta(id)?;
        let name = match meta.get("name") {
            Some(ValueOrContainer::Value(LoroValue::String(s))) => Some(s.to_string()),
            _ => None,
        };
        Ok(name)
    }

    async fn set_name(
        &mut self,
        self_: Resource<HostNode>,
        value: Option<String>,
    ) -> wasmtime::Result<()> {
        let id = self.table.get(&self_)?.tree_id;
        let meta = self.node_meta(id)?;
        value
            .map_or_else(
                || meta.insert("name", LoroValue::Null),
                |s| meta.insert("name", s.as_str()),
            )
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    async fn translation(&mut self, self_: Resource<HostNode>) -> wasmtime::Result<Vec3> {
        let id = self.table.get(&self_)?.tree_id;
        let meta = self.node_meta(id)?;
        let [x, y, z] = read_f64s::<3>(&meta, "translation").unwrap_or([0.0, 0.0, 0.0]);
        #[expect(clippy::cast_possible_truncation)]
        Ok(Vec3 {
            x: x as f32,
            y: y as f32,
            z: z as f32,
        })
    }

    async fn set_translation(
        &mut self,
        self_: Resource<HostNode>,
        value: Vec3,
    ) -> wasmtime::Result<()> {
        let id = self.table.get(&self_)?.tree_id;
        let meta = self.node_meta(id)?;
        write_f64s(
            &meta,
            "translation",
            &[f64::from(value.x), f64::from(value.y), f64::from(value.z)],
        )
    }

    async fn rotation(&mut self, self_: Resource<HostNode>) -> wasmtime::Result<Quat> {
        let id = self.table.get(&self_)?.tree_id;
        let meta = self.node_meta(id)?;
        let [x, y, z, w] = read_f64s::<4>(&meta, "rotation").unwrap_or([0.0, 0.0, 0.0, 1.0]);
        #[expect(clippy::cast_possible_truncation)]
        Ok(Quat {
            x: x as f32,
            y: y as f32,
            z: z as f32,
            w: w as f32,
        })
    }

    async fn set_rotation(
        &mut self,
        self_: Resource<HostNode>,
        value: Quat,
    ) -> wasmtime::Result<()> {
        let id = self.table.get(&self_)?.tree_id;
        let meta = self.node_meta(id)?;
        write_f64s(
            &meta,
            "rotation",
            &[
                f64::from(value.x),
                f64::from(value.y),
                f64::from(value.z),
                f64::from(value.w),
            ],
        )
    }

    async fn scale(&mut self, self_: Resource<HostNode>) -> wasmtime::Result<Vec3> {
        let id = self.table.get(&self_)?.tree_id;
        let meta = self.node_meta(id)?;
        let [x, y, z] = read_f64s::<3>(&meta, "scale").unwrap_or([1.0, 1.0, 1.0]);
        #[expect(clippy::cast_possible_truncation)]
        Ok(Vec3 {
            x: x as f32,
            y: y as f32,
            z: z as f32,
        })
    }

    async fn set_scale(&mut self, self_: Resource<HostNode>, value: Vec3) -> wasmtime::Result<()> {
        let id = self.table.get(&self_)?.tree_id;
        let meta = self.node_meta(id)?;
        write_f64s(
            &meta,
            "scale",
            &[f64::from(value.x), f64::from(value.y), f64::from(value.z)],
        )
    }

    async fn transform(&mut self, self_: Resource<HostNode>) -> wasmtime::Result<Transform> {
        let id = self.table.get(&self_)?.tree_id;
        let meta = self.node_meta(id)?;
        let [tx, ty, tz] = read_f64s::<3>(&meta, "translation").unwrap_or([0.0, 0.0, 0.0]);
        let [rx, ry, rz, rw] = read_f64s::<4>(&meta, "rotation").unwrap_or([0.0, 0.0, 0.0, 1.0]);
        let [sx, sy, sz] = read_f64s::<3>(&meta, "scale").unwrap_or([1.0, 1.0, 1.0]);
        #[expect(clippy::cast_possible_truncation)]
        Ok(Transform {
            translation: Vec3 {
                x: tx as f32,
                y: ty as f32,
                z: tz as f32,
            },
            rotation: Quat {
                x: rx as f32,
                y: ry as f32,
                z: rz as f32,
                w: rw as f32,
            },
            scale: Vec3 {
                x: sx as f32,
                y: sy as f32,
                z: sz as f32,
            },
        })
    }

    async fn set_transform(
        &mut self,
        self_: Resource<HostNode>,
        value: Transform,
    ) -> wasmtime::Result<()> {
        let id = self.table.get(&self_)?.tree_id;
        let meta = self.node_meta(id)?;
        write_f64s(
            &meta,
            "translation",
            &[
                f64::from(value.translation.x),
                f64::from(value.translation.y),
                f64::from(value.translation.z),
            ],
        )?;
        write_f64s(
            &meta,
            "rotation",
            &[
                f64::from(value.rotation.x),
                f64::from(value.rotation.y),
                f64::from(value.rotation.z),
                f64::from(value.rotation.w),
            ],
        )?;
        write_f64s(
            &meta,
            "scale",
            &[
                f64::from(value.scale.x),
                f64::from(value.scale.y),
                f64::from(value.scale.z),
            ],
        )
    }

    async fn global_transform(&mut self, self_: Resource<HostNode>) -> wasmtime::Result<Transform> {
        let id = self.table.get(&self_)?.tree_id;
        let tree = self.node_tree()?;

        // Walk ancestors from self up to root, collecting IDs.
        let mut chain = vec![id];
        let mut current = id;
        while let Some(TreeParentId::Node(parent_id)) = tree.parent(current) {
            chain.push(parent_id);
            current = parent_id;
        }
        chain.reverse(); // root-first order

        // Compose transforms from root down to self.
        let mut combined = BevyTransform::IDENTITY;
        for node_id in chain {
            let meta = self.node_meta(node_id)?;
            combined = combined.mul_transform(bevy_transform(&meta));
        }

        let t = combined.translation;
        let r = combined.rotation;
        let s = combined.scale;
        Ok(Transform {
            translation: Vec3 {
                x: t.x,
                y: t.y,
                z: t.z,
            },
            rotation: Quat {
                x: r.x,
                y: r.y,
                z: r.z,
                w: r.w,
            },
            scale: Vec3 {
                x: s.x,
                y: s.y,
                z: s.z,
            },
        })
    }

    async fn parent(
        &mut self,
        self_: Resource<HostNode>,
    ) -> wasmtime::Result<Option<Resource<HostNode>>> {
        let id = self.table.get(&self_)?.tree_id;
        let tree = self.node_tree()?;
        match tree.parent(id) {
            Some(TreeParentId::Node(parent_id)) => {
                Ok(Some(self.table.push(HostNode { tree_id: parent_id })?))
            }
            _ => Ok(None),
        }
    }

    async fn children(
        &mut self,
        self_: Resource<HostNode>,
    ) -> wasmtime::Result<Vec<Resource<HostNode>>> {
        let id = self.table.get(&self_)?.tree_id;
        let tree = self.node_tree()?;
        let child_ids = tree.children(TreeParentId::Node(id)).unwrap_or_default();
        let mut out = Vec::with_capacity(child_ids.len());
        for tree_id in child_ids {
            out.push(self.table.push(HostNode { tree_id })?);
        }
        Ok(out)
    }

    async fn add_child(
        &mut self,
        self_: Resource<HostNode>,
        child: Resource<HostNode>,
    ) -> wasmtime::Result<()> {
        let parent_id = self.table.get(&self_)?.tree_id;
        let child_id = self.table.get(&child)?.tree_id;
        self.node_tree()?
            .mov(child_id, TreeParentId::Node(parent_id))
            .map_err(|e| anyhow::anyhow!("add child: {e}"))
    }

    async fn remove_child(
        &mut self,
        _self_: Resource<HostNode>,
        child: Resource<HostNode>,
    ) -> wasmtime::Result<()> {
        let child_id = self.table.get(&child)?.tree_id;
        self.node_tree()?
            .mov(child_id, TreeParentId::Root)
            .map_err(|e| anyhow::anyhow!("remove child: {e}"))
    }

    async fn mesh(
        &mut self,
        self_: Resource<HostNode>,
    ) -> wasmtime::Result<Option<Resource<Mesh>>> {
        let id = self.table.get(&self_)?.tree_id;
        let meta = self.node_meta(id)?;
        let Some(ValueOrContainer::Value(LoroValue::I64(i))) = meta.get("mesh") else {
            return Ok(None);
        };
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let index = i as usize;
        Ok(Some(self.table.push(HostMesh { index })?))
    }

    async fn set_mesh(
        &mut self,
        self_: Resource<HostNode>,
        value: Option<Resource<Mesh>>,
    ) -> wasmtime::Result<()> {
        let id = self.table.get(&self_)?.tree_id;
        let meta = self.node_meta(id)?;
        match value {
            Some(res) => {
                #[expect(clippy::cast_possible_wrap)]
                let index = self.table.get(&res)?.index as i64;
                meta.insert("mesh", index)
                    .map_err(|e| anyhow::anyhow!("{e}"))
            }
            None => meta
                .insert("mesh", LoroValue::Null)
                .map_err(|e| anyhow::anyhow!("{e}")),
        }
    }

    async fn material(
        &mut self,
        self_: Resource<HostNode>,
    ) -> wasmtime::Result<Option<Resource<Material>>> {
        let id = self.table.get(&self_)?.tree_id;
        let meta = self.node_meta(id)?;
        let Some(ValueOrContainer::Value(LoroValue::I64(i))) = meta.get("material") else {
            return Ok(None);
        };
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let index = i as usize;
        Ok(Some(self.table.push(HostMaterial { index })?))
    }

    async fn set_material(
        &mut self,
        self_: Resource<HostNode>,
        value: Option<Resource<Material>>,
    ) -> wasmtime::Result<()> {
        let id = self.table.get(&self_)?.tree_id;
        let meta = self.node_meta(id)?;
        match value {
            Some(res) => {
                #[expect(clippy::cast_possible_wrap)]
                let index = self.table.get(&res)?.index as i64;
                meta.insert("material", index)
                    .map_err(|e| anyhow::anyhow!("{e}"))
            }
            None => meta
                .insert("material", LoroValue::Null)
                .map_err(|e| anyhow::anyhow!("{e}")),
        }
    }

    async fn collider(&mut self, self_: Resource<HostNode>) -> wasmtime::Result<Option<Collider>> {
        let id = self.table.get(&self_)?.tree_id;
        let meta = self.node_meta(id)?;
        let Some(ValueOrContainer::Container(Container::Map(cmap))) = meta.get("collider") else {
            return Ok(None);
        };
        let Some(ValueOrContainer::Value(LoroValue::String(shape))) = cmap.get("shape") else {
            return Ok(None);
        };
        let Some(ValueOrContainer::Container(Container::List(size_list))) = cmap.get("size") else {
            return Ok(None);
        };
        let LoroValue::List(items) = size_list.get_deep_value() else {
            return Ok(None);
        };
        let size: Vec<f64> = items
            .iter()
            .filter_map(|v| match v {
                LoroValue::Double(d) => Some(*d),
                _ => None,
            })
            .collect();

        #[expect(clippy::cast_possible_truncation)]
        let collider = match shape.as_str() {
            "cuboid" if size.len() == 3 => Collider::Cuboid(Vec3 {
                x: size[0] as f32,
                y: size[1] as f32,
                z: size[2] as f32,
            }),
            "sphere" if size.len() == 1 => Collider::Sphere(size[0] as f32),
            "capsule" if size.len() == 2 => Collider::Capsule(ColliderCapsule {
                radius: size[0] as f32,
                half_height: size[1] as f32,
            }),
            _ => return Ok(None),
        };
        Ok(Some(collider))
    }

    async fn set_collider(
        &mut self,
        self_: Resource<HostNode>,
        value: Option<Collider>,
    ) -> wasmtime::Result<()> {
        let id = self.table.get(&self_)?.tree_id;
        let meta = self.node_meta(id)?;
        match value {
            None => meta
                .insert("collider", LoroValue::Null)
                .map_err(|e| anyhow::anyhow!("{e}")),
            Some(collider) => {
                let cmap = meta
                    .get_or_create_container("collider", LoroMap::new())
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                let (shape, size): (&str, Vec<f64>) = match collider {
                    Collider::Cuboid(v) => (
                        "cuboid",
                        vec![f64::from(v.x), f64::from(v.y), f64::from(v.z)],
                    ),
                    Collider::Sphere(r) => ("sphere", vec![f64::from(r)]),
                    Collider::Capsule(c) => (
                        "capsule",
                        vec![f64::from(c.radius), f64::from(c.half_height)],
                    ),
                };
                cmap.insert("shape", shape)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                write_f64s(&cmap, "size", &size)
            }
        }
    }

    async fn rigid_body(
        &mut self,
        self_: Resource<HostNode>,
    ) -> wasmtime::Result<Option<RigidBodyKind>> {
        let id = self.table.get(&self_)?.tree_id;
        let meta = self.node_meta(id)?;
        let Some(ValueOrContainer::Container(Container::Map(rbmap))) = meta.get("rigid_body")
        else {
            return Ok(None);
        };
        let Some(ValueOrContainer::Value(LoroValue::String(kind))) = rbmap.get("kind") else {
            return Ok(None);
        };
        let rb = match kind.as_str() {
            "dynamic" => RigidBodyKind::Dynamic,
            "fixed" => RigidBodyKind::Fixed,
            "kinematic" => RigidBodyKind::Kinematic,
            _ => return Ok(None),
        };
        Ok(Some(rb))
    }

    async fn set_rigid_body(
        &mut self,
        self_: Resource<HostNode>,
        value: Option<RigidBodyKind>,
    ) -> wasmtime::Result<()> {
        let id = self.table.get(&self_)?.tree_id;
        let meta = self.node_meta(id)?;
        match value {
            None => meta
                .insert("rigid_body", LoroValue::Null)
                .map_err(|e| anyhow::anyhow!("{e}")),
            Some(kind) => {
                let rbmap = meta
                    .get_or_create_container("rigid_body", LoroMap::new())
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                let kind_str = match kind {
                    RigidBodyKind::Dynamic => "dynamic",
                    RigidBodyKind::Fixed => "fixed",
                    RigidBodyKind::Kinematic => "kinematic",
                };
                rbmap
                    .insert("kind", kind_str)
                    .map_err(|e| anyhow::anyhow!("{e}"))
            }
        }
    }

    async fn drop(&mut self, rep: Resource<HostNode>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
