use std::sync::{Arc, atomic::Ordering};

use bevy::prelude::{Entity, Transform as BevyTransform, World};
use bevy_hsd::cache::{MaterialInner, MeshInner, NodeInner};
use bevy_hsd::data::HsdCollider;
use bevy_hsd::hydrate::compile::node::{HsdNodeColliderSet, HsdNodeRigidBodySet};
use bytes::Bytes;
use wasmtime::bail;
use wasmtime::component::Resource;
use wired_records::HydratedHash;

use super::bindings::wired::scene::types::{
    Collider, ColliderCapsule, ColliderCylinder, ColliderTrimesh, Material, Mesh, Quat,
    RigidBodyKind, Transform, Vec3,
};
use super::{WiredSceneRt, material::HostMaterial, mesh::HostMesh};
use crate::core_ops;

pub struct HostNode {
    pub inner: Arc<NodeInner>,
    pub can_read: bool,
    pub can_write: bool,
    pub doc_entity: Entity,
}

impl Clone for HostNode {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            can_read: self.can_read,
            can_write: self.can_write,
            doc_entity: self.doc_entity,
        }
    }
}

impl super::bindings::wired::scene::types::HostNode for WiredSceneRt {
    async fn id(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
    ) -> wasmtime::Result<String> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner.id.to_string())
    }
    async fn clone(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
    ) -> wasmtime::Result<wasmtime::component::Resource<HostNode>> {
        let inner = self.table.get(&self_)?.clone();
        let node = self.table.push(inner)?;
        Ok(node)
    }

    async fn name(&mut self, self_: Resource<HostNode>) -> wasmtime::Result<Option<String>> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let state = inner.state.lock().expect("node state lock");
        Ok(state.name.clone())
    }

    async fn set_name(
        &mut self,
        self_: Resource<HostNode>,
        value: Option<String>,
    ) -> wasmtime::Result<()> {
        let (inner, doc) = {
            let n = self.table.get(&self_)?;
            (Arc::clone(&n.inner), n.doc_entity)
        };
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::node::set_name(&inner, doc, value, &mut queue);
        Ok(())
    }

    async fn translation(&mut self, self_: Resource<HostNode>) -> wasmtime::Result<Vec3> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let t = inner
            .state
            .lock()
            .expect("node state lock")
            .transform
            .translation;
        Ok(Vec3 {
            x: t.x,
            y: t.y,
            z: t.z,
        })
    }

    async fn set_translation(
        &mut self,
        self_: Resource<HostNode>,
        value: Vec3,
    ) -> wasmtime::Result<()> {
        let (inner, doc) = {
            let n = self.table.get(&self_)?;
            (Arc::clone(&n.inner), n.doc_entity)
        };
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::node::set_translation(&inner, doc, value.x, value.y, value.z, &mut queue);
        Ok(())
    }

    async fn rotation(&mut self, self_: Resource<HostNode>) -> wasmtime::Result<Quat> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let r = inner
            .state
            .lock()
            .expect("node state lock")
            .transform
            .rotation;
        Ok(Quat {
            x: r.x,
            y: r.y,
            z: r.z,
            w: r.w,
        })
    }

    async fn set_rotation(
        &mut self,
        self_: Resource<HostNode>,
        value: Quat,
    ) -> wasmtime::Result<()> {
        let (inner, doc) = {
            let n = self.table.get(&self_)?;
            (Arc::clone(&n.inner), n.doc_entity)
        };
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::node::set_rotation(&inner, doc, value.x, value.y, value.z, value.w, &mut queue);
        Ok(())
    }

    async fn scale(&mut self, self_: Resource<HostNode>) -> wasmtime::Result<Vec3> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let s = inner.state.lock().expect("node state lock").transform.scale;
        Ok(Vec3 {
            x: s.x,
            y: s.y,
            z: s.z,
        })
    }

    async fn set_scale(&mut self, self_: Resource<HostNode>, value: Vec3) -> wasmtime::Result<()> {
        let (inner, doc) = {
            let n = self.table.get(&self_)?;
            (Arc::clone(&n.inner), n.doc_entity)
        };
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::node::set_scale(&inner, doc, value.x, value.y, value.z, &mut queue);
        Ok(())
    }

    async fn transform(&mut self, self_: Resource<HostNode>) -> wasmtime::Result<Transform> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let state = inner.state.lock().expect("node state lock");
        let t = state.transform.translation;
        let r = state.transform.rotation;
        let s = state.transform.scale;
        drop(state);
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

    async fn set_transform(
        &mut self,
        self_: Resource<HostNode>,
        value: Transform,
    ) -> wasmtime::Result<()> {
        let (inner, doc) = {
            let n = self.table.get(&self_)?;
            (Arc::clone(&n.inner), n.doc_entity)
        };
        let new_transform = BevyTransform {
            translation: bevy::math::Vec3::new(
                value.translation.x,
                value.translation.y,
                value.translation.z,
            ),
            rotation: bevy::math::Quat::from_xyzw(
                value.rotation.x,
                value.rotation.y,
                value.rotation.z,
                value.rotation.w,
            ),
            scale: bevy::math::Vec3::new(value.scale.x, value.scale.y, value.scale.z),
        };
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::node::set_transform(&inner, doc, new_transform, &mut queue);
        Ok(())
    }

    async fn global_transform(&mut self, self_: Resource<HostNode>) -> wasmtime::Result<Transform> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let gt = inner
            .state
            .lock()
            .expect("node state lock")
            .global_transform;
        let (s, r, t) = gt.to_scale_rotation_translation();
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
        let (inner, can_read, can_write, doc_entity) = {
            let n = self.table.get(&self_)?;
            (Arc::clone(&n.inner), n.can_read, n.can_write, n.doc_entity)
        };
        let parent_inner = {
            let state = inner.state.lock().expect("node state lock");
            state.parent.as_ref().and_then(std::sync::Weak::upgrade)
        };
        match parent_inner {
            Some(pi) => Ok(Some(self.table.push(HostNode {
                inner: pi,
                can_read,
                can_write,
                doc_entity,
            })?)),
            None => Ok(None),
        }
    }

    async fn children(
        &mut self,
        self_: Resource<HostNode>,
    ) -> wasmtime::Result<Vec<Resource<HostNode>>> {
        let (inner, can_read, can_write, doc_entity) = {
            let n = self.table.get(&self_)?;
            (Arc::clone(&n.inner), n.can_read, n.can_write, n.doc_entity)
        };
        let children: Vec<Arc<NodeInner>> = {
            let state = inner.state.lock().expect("node state lock");
            state.children.clone()
        };
        let mut out = Vec::with_capacity(children.len());
        for child in children {
            out.push(self.table.push(HostNode {
                inner: child,
                can_read,
                can_write,
                doc_entity,
            })?);
        }
        Ok(out)
    }

    async fn add_child(
        &mut self,
        self_: Resource<HostNode>,
        child: Resource<HostNode>,
    ) -> wasmtime::Result<()> {
        let (parent_inner, parent_can_write, doc) = {
            let n = self.table.get(&self_)?;
            (Arc::clone(&n.inner), n.can_write, n.doc_entity)
        };
        let (child_inner, child_can_write) = {
            let n = self.table.get(&child)?;
            (Arc::clone(&n.inner), n.can_write)
        };
        if !parent_can_write || !child_can_write {
            bail!("hsd write permission required")
        }
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::node::add_child(&parent_inner, &child_inner, doc, &mut queue);
        Ok(())
    }

    async fn remove_child(
        &mut self,
        _self_: Resource<HostNode>,
        child: Resource<HostNode>,
    ) -> wasmtime::Result<()> {
        let (child_inner, can_write, doc) = {
            let n = self.table.get(&child)?;
            (Arc::clone(&n.inner), n.can_write, n.doc_entity)
        };
        if !can_write {
            bail!("hsd write permission required")
        }
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::node::remove_child(&child_inner, doc, &mut queue);
        Ok(())
    }

    async fn mesh(
        &mut self,
        self_: Resource<HostNode>,
    ) -> wasmtime::Result<Option<Resource<Mesh>>> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let state = inner.state.lock().expect("node state lock");
        let Some(mesh_id) = &state.mesh else {
            return Ok(None);
        };
        let mesh_inner: Option<Arc<MeshInner>> = {
            let meshes = self.registry.meshes.lock().expect("meshes lock");
            meshes.get(mesh_id).cloned()
        };
        drop(state);
        let Some(mesh_inner) = mesh_inner else {
            return Ok(None);
        };
        Ok(Some(self.table.push(HostMesh {
            inner: mesh_inner,
            can_read: true,
            can_write: true,
        })?))
    }

    async fn set_mesh(
        &mut self,
        self_: Resource<HostNode>,
        value: Option<Resource<Mesh>>,
    ) -> wasmtime::Result<()> {
        let (node_inner, doc) = {
            let n = self.table.get(&self_)?;
            (Arc::clone(&n.inner), n.doc_entity)
        };
        let mesh_id = match &value {
            Some(res) => Some(self.table.get(res)?.inner.id.clone()),
            None => None,
        };
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::node::set_mesh(&node_inner, doc, mesh_id, &mut queue);
        Ok(())
    }

    async fn material(
        &mut self,
        self_: Resource<HostNode>,
    ) -> wasmtime::Result<Option<Resource<Material>>> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let state = inner.state.lock().expect("node state lock");
        let Some(mat_id) = &state.material else {
            return Ok(None);
        };
        let mat_inner: Option<Arc<MaterialInner>> = {
            let mats = self.registry.materials.lock().expect("materials lock");
            mats.get(mat_id).cloned()
        };
        drop(state);
        let Some(mat_inner) = mat_inner else {
            return Ok(None);
        };
        Ok(Some(self.table.push(HostMaterial {
            inner: mat_inner,
            can_read: true,
            can_write: true,
        })?))
    }

    async fn set_material(
        &mut self,
        self_: Resource<HostNode>,
        value: Option<Resource<Material>>,
    ) -> wasmtime::Result<()> {
        let (node_inner, doc) = {
            let n = self.table.get(&self_)?;
            (Arc::clone(&n.inner), n.doc_entity)
        };
        let mat_id = match &value {
            Some(res) => Some(self.table.get(res)?.inner.id.clone()),
            None => None,
        };
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::node::set_material(&node_inner, doc, mat_id, &mut queue);
        Ok(())
    }

    async fn collider(&mut self, self_: Resource<HostNode>) -> wasmtime::Result<Option<Collider>> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let c = inner
            .state
            .lock()
            .expect("node state lock")
            .collider
            .clone();
        let Some(c) = c else {
            return Ok(None);
        };
        let collider = match &c {
            HsdCollider::Capsule { radius, height } => Collider::Capsule(ColliderCapsule {
                height: *height as f32,
                radius: *radius as f32,
            }),
            HsdCollider::ConvexHull(hash) => {
                let blobs = self
                    .blobs
                    .as_ref()
                    .ok_or_else(|| wasmtime::Error::msg("no blob store"))?;
                let bytes = blobs
                    .get_bytes(hash.0)
                    .await
                    .map_err(|e| wasmtime::Error::msg(e.to_string()))?;
                let pts: &[f32] = bytemuck::cast_slice(&bytes);
                Collider::ConvexHull(pts.to_vec())
            }
            HsdCollider::Cuboid { x, y, z } => Collider::Cuboid(Vec3 {
                x: *x as f32,
                y: *y as f32,
                z: *z as f32,
            }),
            HsdCollider::Cylinder { radius, height } => Collider::Cylinder(ColliderCylinder {
                height: *height as f32,
                radius: *radius as f32,
            }),
            HsdCollider::Sphere(r) => Collider::Sphere(*r as f32),
            HsdCollider::Trimesh { vertices, indices } => {
                let blobs = self
                    .blobs
                    .as_ref()
                    .ok_or_else(|| wasmtime::Error::msg("no blob store"))?;
                let vbytes = blobs
                    .get_bytes(vertices.0)
                    .await
                    .map_err(|e| wasmtime::Error::msg(e.to_string()))?;
                let ibytes = blobs
                    .get_bytes(indices.0)
                    .await
                    .map_err(|e| wasmtime::Error::msg(e.to_string()))?;
                let verts: &[f32] = bytemuck::cast_slice(&vbytes);
                let idxs: &[u32] = bytemuck::cast_slice(&ibytes);
                Collider::Trimesh(ColliderTrimesh {
                    vertices: verts.to_vec(),
                    indices: idxs.to_vec(),
                })
            }
        };
        Ok(Some(collider))
    }

    async fn set_collider(
        &mut self,
        self_: Resource<HostNode>,
        value: Option<Collider>,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        if inner.is_virtual {
            return Ok(());
        }
        let hsd_collider = match value {
            None => None,
            Some(c) => Some(match c {
                Collider::Capsule(cap) => {
                    validate_positive(cap.radius, "capsule radius")?;
                    validate_nonneg(cap.height, "capsule height")?;
                    HsdCollider::Capsule {
                        height: f64::from(cap.height),
                        radius: f64::from(cap.radius),
                    }
                }
                Collider::ConvexHull(pts) => {
                    let actor = self
                        .actor
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("no actor"))?;
                    let bytes = Bytes::from(bytemuck::cast_slice::<f32, u8>(&pts).to_vec());
                    let hash = actor
                        .upload_blob(bytes)
                        .await
                        .map_err(|e| wasmtime::Error::msg(e.to_string()))?;
                    HsdCollider::ConvexHull(HydratedHash(hash))
                }
                Collider::Cuboid(v) => {
                    validate_positive(v.x, "cuboid x")?;
                    validate_positive(v.y, "cuboid y")?;
                    validate_positive(v.z, "cuboid z")?;
                    HsdCollider::Cuboid {
                        x: f64::from(v.x),
                        y: f64::from(v.y),
                        z: f64::from(v.z),
                    }
                }
                Collider::Cylinder(cyl) => {
                    validate_positive(cyl.radius, "cylinder radius")?;
                    validate_nonneg(cyl.height, "cylinder height")?;
                    HsdCollider::Cylinder {
                        height: f64::from(cyl.height),
                        radius: f64::from(cyl.radius),
                    }
                }
                Collider::Sphere(r) => {
                    validate_positive(r, "sphere radius")?;
                    HsdCollider::Sphere(f64::from(r))
                }
                Collider::Trimesh(t) => {
                    let actor = self
                        .actor
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("no actor"))?;
                    let vbytes = Bytes::from(bytemuck::cast_slice::<f32, u8>(&t.vertices).to_vec());
                    let ibytes = Bytes::from(bytemuck::cast_slice::<u32, u8>(&t.indices).to_vec());
                    let vhash = actor
                        .upload_blob(vbytes)
                        .await
                        .map_err(|e| wasmtime::Error::msg(e.to_string()))?;
                    let ihash = actor
                        .upload_blob(ibytes)
                        .await
                        .map_err(|e| wasmtime::Error::msg(e.to_string()))?;
                    HsdCollider::Trimesh {
                        vertices: HydratedHash(vhash),
                        indices: HydratedHash(ihash),
                    }
                }
            }),
        };
        inner
            .state
            .lock()
            .expect("node state lock")
            .collider
            .clone_from(&hsd_collider);
        let doc = {
            let n = self.table.get(&self_)?;
            n.doc_entity
        };
        let id = inner.id.clone();
        self.push_command(move |world: &mut World| {
            world.trigger(HsdNodeColliderSet {
                doc,
                id,
                collider: hsd_collider,
            });
        });
        Ok(())
    }

    async fn rigid_body(
        &mut self,
        self_: Resource<HostNode>,
    ) -> wasmtime::Result<Option<RigidBodyKind>> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let Some(rb) = &inner.state.lock().expect("node state lock").rigid_body else {
            return Ok(None);
        };
        let kind = match rb.kind.as_str() {
            "dynamic" => RigidBodyKind::Dynamic,
            "fixed" => RigidBodyKind::Fixed,
            "kinematic" => RigidBodyKind::Kinematic,
            _ => return Ok(None),
        };
        Ok(Some(kind))
    }

    async fn set_rigid_body(
        &mut self,
        self_: Resource<HostNode>,
        value: Option<RigidBodyKind>,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        if inner.is_virtual {
            return Ok(());
        }
        let rb = value.map(|kind| {
            let kind_str = match kind {
                RigidBodyKind::Dynamic => "dynamic",
                RigidBodyKind::Fixed => "fixed",
                RigidBodyKind::Kinematic => "kinematic",
            };
            bevy_hsd::data::HsdRigidBody {
                kind: kind_str.into(),
                ..Default::default()
            }
        });
        inner
            .state
            .lock()
            .expect("node state lock")
            .rigid_body
            .clone_from(&rb);
        let doc = {
            let n = self.table.get(&self_)?;
            n.doc_entity
        };
        let id = inner.id.clone();
        self.push_command(move |world: &mut World| {
            world.trigger(HsdNodeRigidBodySet {
                doc,
                id,
                rigid_body: rb,
            });
        });
        Ok(())
    }

    async fn sync(&mut self, self_: Resource<HostNode>) -> wasmtime::Result<bool> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner.sync.load(Ordering::Relaxed))
    }

    async fn set_sync(&mut self, self_: Resource<HostNode>, value: bool) -> wasmtime::Result<()> {
        let (inner, can_write) = {
            let n = self.table.get(&self_)?;
            (Arc::clone(&n.inner), n.can_write)
        };
        if inner.is_virtual {
            return Ok(());
        }
        if value && !can_write {
            bail!("hsd write permission required")
        }
        inner.sync.store(value, Ordering::Relaxed);
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<HostNode>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}

fn validate_positive(v: f32, name: &str) -> wasmtime::Result<()> {
    if v.is_finite() && v > 0.0 {
        Ok(())
    } else {
        bail!("{name} must be finite and positive, got {v}")
    }
}

fn validate_nonneg(v: f32, name: &str) -> wasmtime::Result<()> {
    if v.is_finite() && v >= 0.0 {
        Ok(())
    } else {
        bail!("{name} must be finite and non-negative, got {v}")
    }
}
