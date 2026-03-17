use std::sync::{Arc, atomic::Ordering};

use bevy::prelude::Transform as BevyTransform;
use bevy_hsd::cache::{MaterialInner, MeshInner, NodeInner};
use bevy_hsd::data::HsdCollider;
use bevy_hsd::hydrate::events::ScriptQueuedEvent;
use bytes::Bytes;
use wasmtime::component::Resource;
use wired_schemas::HydratedHash;

use super::bindings::wired::scene::types::{
    Collider, ColliderCapsule, ColliderCylinder, ColliderTrimesh, Material, Mesh, Quat,
    RigidBodyKind, Transform, Vec3,
};
use super::{WiredSceneRt, material::HostMaterial, mesh::HostMesh};

#[derive(Clone)]
pub struct HostNode {
    pub inner: Arc<NodeInner>,
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
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        if inner.is_virtual {
            return Ok(());
        }
        inner
            .state
            .lock()
            .expect("node state lock")
            .name
            .clone_from(&value);
        if inner.sync.load(Ordering::Relaxed) {
            inner.hsd_changes.lock().expect("hsd_changes lock").name = Some(value);
        } else {
            inner.dirty.lock().expect("dirty lock").name = true;
        }
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
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        if inner.is_virtual {
            return Ok(());
        }
        inner
            .state
            .lock()
            .expect("node state lock")
            .transform
            .translation = bevy::math::Vec3::new(value.x, value.y, value.z);
        if inner.sync.load(Ordering::Relaxed) {
            inner
                .hsd_changes
                .lock()
                .expect("hsd_changes lock")
                .translation = Some([f64::from(value.x), f64::from(value.y), f64::from(value.z)]);
        } else {
            inner.dirty.lock().expect("dirty lock").transform = true;
        }
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
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        if inner.is_virtual {
            return Ok(());
        }
        inner
            .state
            .lock()
            .expect("node state lock")
            .transform
            .rotation = bevy::math::Quat::from_xyzw(value.x, value.y, value.z, value.w);
        if inner.sync.load(Ordering::Relaxed) {
            inner.hsd_changes.lock().expect("hsd_changes lock").rotation = Some([
                f64::from(value.x),
                f64::from(value.y),
                f64::from(value.z),
                f64::from(value.w),
            ]);
        } else {
            inner.dirty.lock().expect("dirty lock").transform = true;
        }
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
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        if inner.is_virtual {
            return Ok(());
        }
        inner.state.lock().expect("node state lock").transform.scale =
            bevy::math::Vec3::new(value.x, value.y, value.z);
        if inner.sync.load(Ordering::Relaxed) {
            inner.hsd_changes.lock().expect("hsd_changes lock").scale =
                Some([f64::from(value.x), f64::from(value.y), f64::from(value.z)]);
        } else {
            inner.dirty.lock().expect("dirty lock").transform = true;
        }
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
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        if inner.is_virtual {
            return Ok(());
        }
        inner.state.lock().expect("node state lock").transform = BevyTransform {
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
        if inner.sync.load(Ordering::Relaxed) {
            let mut ch = inner.hsd_changes.lock().expect("hsd_changes lock");
            ch.translation = Some([
                f64::from(value.translation.x),
                f64::from(value.translation.y),
                f64::from(value.translation.z),
            ]);
            ch.rotation = Some([
                f64::from(value.rotation.x),
                f64::from(value.rotation.y),
                f64::from(value.rotation.z),
                f64::from(value.rotation.w),
            ]);
            ch.scale = Some([
                f64::from(value.scale.x),
                f64::from(value.scale.y),
                f64::from(value.scale.z),
            ]);
        } else {
            inner.dirty.lock().expect("dirty lock").transform = true;
        }
        Ok(())
    }

    async fn global_transform(&mut self, self_: Resource<HostNode>) -> wasmtime::Result<Transform> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let gt = inner
            .state
            .lock()
            .expect("node state lock")
            .global_transform;
        let t = gt.translation();
        let r = gt.to_scale_rotation_translation().1;
        let s = gt.to_scale_rotation_translation().0;
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
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let parent_inner = {
            let state = inner.state.lock().expect("node state lock");
            state.parent.as_ref().and_then(std::sync::Weak::upgrade)
        };
        match parent_inner {
            Some(pi) => Ok(Some(self.table.push(HostNode { inner: pi })?)),
            None => Ok(None),
        }
    }

    async fn children(
        &mut self,
        self_: Resource<HostNode>,
    ) -> wasmtime::Result<Vec<Resource<HostNode>>> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let children: Vec<Arc<NodeInner>> = {
            let state = inner.state.lock().expect("node state lock");
            state.children.clone()
        };
        let mut out = Vec::with_capacity(children.len());
        for child in children {
            out.push(self.table.push(HostNode { inner: child })?);
        }
        Ok(out)
    }

    async fn add_child(
        &mut self,
        self_: Resource<HostNode>,
        child: Resource<HostNode>,
    ) -> wasmtime::Result<()> {
        let parent_inner = Arc::clone(&self.table.get(&self_)?.inner);
        let child_inner = Arc::clone(&self.table.get(&child)?.inner);

        {
            let mut parent_state = parent_inner.state.lock().expect("parent state lock");
            if !parent_state.children.iter().any(|c| c.id == child_inner.id) {
                parent_state.children.push(Arc::clone(&child_inner));
            }
        }
        {
            let mut child_state = child_inner.state.lock().expect("child state lock");
            child_state.parent = Some(Arc::downgrade(&parent_inner));
        }

        self.push_script_event(ScriptQueuedEvent::NodeParentSet {
            id: child_inner.id.clone(),
            parent: Some(parent_inner.id.clone()),
        });
        Ok(())
    }

    async fn remove_child(
        &mut self,
        _self_: Resource<HostNode>,
        child: Resource<HostNode>,
    ) -> wasmtime::Result<()> {
        let child_inner = Arc::clone(&self.table.get(&child)?.inner);

        let parent_inner = {
            let child_state = child_inner.state.lock().expect("child state lock");
            child_state
                .parent
                .as_ref()
                .and_then(std::sync::Weak::upgrade)
        };
        if let Some(pi) = &parent_inner {
            pi.state
                .lock()
                .expect("parent state lock")
                .children
                .retain(|c| c.id != child_inner.id);
        }
        child_inner.state.lock().expect("child state lock").parent = None;

        self.push_script_event(ScriptQueuedEvent::NodeParentSet {
            id: child_inner.id.clone(),
            parent: None,
        });
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
        Ok(Some(self.table.push(HostMesh { inner: mesh_inner })?))
    }

    async fn set_mesh(
        &mut self,
        self_: Resource<HostNode>,
        value: Option<Resource<Mesh>>,
    ) -> wasmtime::Result<()> {
        let node_inner = Arc::clone(&self.table.get(&self_)?.inner);
        if node_inner.is_virtual {
            return Ok(());
        }
        let mesh_id = match &value {
            Some(res) => Some(self.table.get(res)?.inner.id.clone()),
            None => None,
        };
        node_inner
            .state
            .lock()
            .expect("node state lock")
            .mesh
            .clone_from(&mesh_id);
        if node_inner.sync.load(Ordering::Relaxed) {
            node_inner
                .hsd_changes
                .lock()
                .expect("hsd_changes lock")
                .mesh = Some(mesh_id);
        } else {
            node_inner.dirty.lock().expect("dirty lock").mesh = true;
        }
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
        Ok(Some(self.table.push(HostMaterial { inner: mat_inner })?))
    }

    async fn set_material(
        &mut self,
        self_: Resource<HostNode>,
        value: Option<Resource<Material>>,
    ) -> wasmtime::Result<()> {
        let node_inner = Arc::clone(&self.table.get(&self_)?.inner);
        if node_inner.is_virtual {
            return Ok(());
        }
        let mat_id = match &value {
            Some(res) => Some(self.table.get(res)?.inner.id.clone()),
            None => None,
        };
        node_inner
            .state
            .lock()
            .expect("node state lock")
            .material
            .clone_from(&mat_id);
        if node_inner.sync.load(Ordering::Relaxed) {
            node_inner
                .hsd_changes
                .lock()
                .expect("hsd_changes lock")
                .material = Some(mat_id);
        } else {
            node_inner.dirty.lock().expect("dirty lock").material = true;
        }
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
        #[expect(clippy::cast_possible_truncation)]
        let collider = match &c {
            HsdCollider::Capsule { radius, height } => Collider::Capsule(ColliderCapsule {
                height: *height as f32,
                radius: *radius as f32,
            }),
            HsdCollider::ConvexHull(hash) => {
                let blobs = self
                    .blobs
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("no blob store"))?;
                let bytes = blobs
                    .get_bytes(hash.0)
                    .await
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
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
                    .ok_or_else(|| anyhow::anyhow!("no blob store"))?;
                let vbytes = blobs
                    .get_bytes(vertices.0)
                    .await
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                let ibytes = blobs
                    .get_bytes(indices.0)
                    .await
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
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
                        .ok_or_else(|| anyhow::anyhow!("no actor"))?;
                    let bytes = Bytes::from(bytemuck::cast_slice::<f32, u8>(&pts).to_vec());
                    let hash = actor
                        .upload_blob(bytes)
                        .await
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
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
                        .ok_or_else(|| anyhow::anyhow!("no actor"))?;
                    let vbytes = Bytes::from(bytemuck::cast_slice::<f32, u8>(&t.vertices).to_vec());
                    let ibytes = Bytes::from(bytemuck::cast_slice::<u32, u8>(&t.indices).to_vec());
                    let vhash = actor
                        .upload_blob(vbytes)
                        .await
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                    let ihash = actor
                        .upload_blob(ibytes)
                        .await
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
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
        inner.dirty.lock().expect("dirty lock").collider = true;
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
        inner.dirty.lock().expect("dirty lock").rigid_body = true;
        Ok(())
    }

    async fn sync(&mut self, self_: Resource<HostNode>) -> wasmtime::Result<bool> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner.sync.load(Ordering::Relaxed))
    }

    async fn set_sync(&mut self, self_: Resource<HostNode>, value: bool) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        if inner.is_virtual {
            return Ok(());
        }
        if value {
            self.check_hsd_write()?;
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
        Err(anyhow::anyhow!(
            "{name} must be finite and positive, got {v}"
        ))
    }
}

fn validate_nonneg(v: f32, name: &str) -> wasmtime::Result<()> {
    if v.is_finite() && v >= 0.0 {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "{name} must be finite and non-negative, got {v}"
        ))
    }
}
