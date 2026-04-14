use std::sync::{Arc, atomic::Ordering};

use bevy::prelude::Transform as BevyTransform;
use bevy_hsd::{
    cache::{MaterialInner, MeshInner, NodeInner, SceneRegistryInner},
    hydrate::compile::node::HsdDocTransformSet,
};
use loro::LoroDoc;
use wasmtime::{bail, component::Resource};
use wds::surg::acl::Acl;

use super::bindings::wired::scene::types::{Document, Material, Mesh, Quat, Transform, Vec3};
use super::{WiredSceneRt, material::HostMaterial, mesh::HostMesh, node::HostNode};
use crate::core_ops;

pub struct HostDocument {
    pub id: blake3::Hash,
    pub registry: Arc<SceneRegistryInner>,
    pub lor_doc: Option<Arc<LoroDoc>>,
    pub is_public: bool,
    pub can_read: bool,
    pub can_write: bool,
}

impl Clone for HostDocument {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            registry: Arc::clone(&self.registry),
            lor_doc: self.lor_doc.as_ref().map(Arc::clone),
            is_public: self.is_public,
            can_read: self.can_read,
            can_write: self.can_write,
        }
    }
}

impl super::bindings::wired::scene::types::HostDocument for WiredSceneRt {
    async fn clone(
        &mut self,
        self_: Resource<HostDocument>,
    ) -> wasmtime::Result<Resource<HostDocument>> {
        let inner = self.table.get(&self_)?.clone();
        Ok(self.table.push(inner)?)
    }

    async fn create_material(
        &mut self,
        self_: Resource<Document>,
    ) -> wasmtime::Result<Resource<Material>> {
        let (doc_id, registry) = {
            let d = self.table.get(&self_)?;
            if !d.can_write {
                bail!("hsd write permission required")
            }
            (d.id, Arc::clone(&d.registry))
        };
        let inner = {
            let mut queue = self.command_queue.lock().expect("cmd queue lock");
            core_ops::document::create_material(&registry, doc_id, &mut queue)
        };
        Ok(self.table.push(HostMaterial {
            inner,
            can_read: true,
            can_write: true,
            doc_id,
        })?)
    }

    async fn create_mesh(&mut self, self_: Resource<Document>) -> wasmtime::Result<Resource<Mesh>> {
        let (doc_id, registry) = {
            let d = self.table.get(&self_)?;
            if !d.can_write {
                bail!("hsd write permission required")
            }
            (d.id, Arc::clone(&d.registry))
        };
        let inner = {
            let mut queue = self.command_queue.lock().expect("cmd queue lock");
            core_ops::document::create_mesh(&registry, doc_id, &mut queue)
        };
        Ok(self.table.push(HostMesh {
            inner,
            can_read: true,
            can_write: true,
            doc_id,
        })?)
    }

    async fn create_node(
        &mut self,
        self_: Resource<Document>,
    ) -> wasmtime::Result<Resource<HostNode>> {
        let (doc_id, registry) = {
            let d = self.table.get(&self_)?;
            if !d.can_write {
                bail!("hsd write permission required")
            }
            (d.id, Arc::clone(&d.registry))
        };
        let inner = {
            let mut queue = self.command_queue.lock().expect("cmd queue lock");
            core_ops::document::create_node(&registry, doc_id, &mut queue)
        };
        Ok(self.table.push(HostNode {
            inner,
            can_read: true,
            can_write: true,
            doc_id,
        })?)
    }

    async fn roots(
        &mut self,
        self_: Resource<Document>,
    ) -> wasmtime::Result<Vec<Resource<HostNode>>> {
        let (registry, can_read, can_write, doc_id) = self.get_doc_read(&self_)?;
        let nodes: Vec<Arc<NodeInner>> = {
            let all = registry.nodes.lock().expect("nodes lock");
            all.iter()
                .filter(|n| {
                    n.state
                        .lock()
                        .expect("node state lock")
                        .parent
                        .as_ref()
                        .is_none_or(|w| w.upgrade().is_none())
                })
                .cloned()
                .collect()
        };
        let mut out = Vec::with_capacity(nodes.len());
        for inner in nodes {
            out.push(self.table.push(HostNode {
                inner,
                can_read,
                can_write,
                doc_id,
            })?);
        }
        Ok(out)
    }

    async fn nodes(
        &mut self,
        self_: Resource<Document>,
    ) -> wasmtime::Result<Vec<Resource<HostNode>>> {
        let (registry, can_read, can_write, doc_id) = self.get_doc_read(&self_)?;
        let nodes: Vec<Arc<NodeInner>> = registry.nodes.lock().expect("nodes lock").clone();
        let mut out = Vec::with_capacity(nodes.len());
        for inner in nodes {
            out.push(self.table.push(HostNode {
                inner,
                can_read,
                can_write,
                doc_id,
            })?);
        }
        Ok(out)
    }

    async fn meshes(
        &mut self,
        self_: Resource<Document>,
    ) -> wasmtime::Result<Vec<Resource<HostMesh>>> {
        let (registry, can_read, can_write, doc_id) = self.get_doc_read(&self_)?;
        let inners: Vec<Arc<MeshInner>> = registry
            .meshes
            .lock()
            .expect("meshes lock")
            .values()
            .cloned()
            .collect();
        let mut out = Vec::with_capacity(inners.len());
        for inner in inners {
            out.push(self.table.push(HostMesh {
                inner,
                can_read,
                can_write,
                doc_id,
            })?);
        }
        Ok(out)
    }

    async fn materials(
        &mut self,
        self_: Resource<Document>,
    ) -> wasmtime::Result<Vec<Resource<HostMaterial>>> {
        let (registry, can_read, can_write, doc_id) = self.get_doc_read(&self_)?;
        let inners: Vec<Arc<MaterialInner>> = registry
            .materials
            .lock()
            .expect("materials lock")
            .values()
            .cloned()
            .collect();
        let mut out = Vec::with_capacity(inners.len());
        for inner in inners {
            out.push(self.table.push(HostMaterial {
                inner,
                can_read,
                can_write,
                doc_id,
            })?);
        }
        Ok(out)
    }

    async fn remove_node(
        &mut self,
        self_: Resource<Document>,
        value: Resource<HostNode>,
    ) -> wasmtime::Result<()> {
        let (doc_id, registry) = {
            let d = self.table.get(&self_)?;
            if !d.can_write {
                bail!("hsd write permission required")
            }
            (d.id, Arc::clone(&d.registry))
        };
        let inner = Arc::clone(&self.table.get(&value)?.inner);
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::document::remove_node(&inner, &registry, doc_id, &mut queue);
        Ok(())
    }

    async fn remove_mesh(
        &mut self,
        self_: Resource<Document>,
        value: Resource<HostMesh>,
    ) -> wasmtime::Result<()> {
        let (doc_id, registry) = {
            let d = self.table.get(&self_)?;
            if !d.can_write {
                bail!("hsd write permission required")
            }
            (d.id, Arc::clone(&d.registry))
        };
        let inner = Arc::clone(&self.table.get(&value)?.inner);
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::document::remove_mesh(&inner, &registry, doc_id, &mut queue);
        Ok(())
    }

    async fn remove_material(
        &mut self,
        self_: Resource<Document>,
        value: Resource<HostMaterial>,
    ) -> wasmtime::Result<()> {
        let (doc_id, registry) = {
            let d = self.table.get(&self_)?;
            if !d.can_write {
                bail!("hsd write permission required")
            }
            (d.id, Arc::clone(&d.registry))
        };
        let inner = Arc::clone(&self.table.get(&value)?.inner);
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::document::remove_material(&inner, &registry, doc_id, &mut queue);
        Ok(())
    }

    async fn sync(&mut self, self_: Resource<Document>) -> wasmtime::Result<bool> {
        let registry = Arc::clone(&self.table.get(&self_)?.registry);
        Ok(registry.doc_sync.load(Ordering::Relaxed))
    }

    async fn set_sync(&mut self, self_: Resource<Document>, value: bool) -> wasmtime::Result<()> {
        let (registry, can_write) = {
            let d = self.table.get(&self_)?;
            (Arc::clone(&d.registry), d.can_write)
        };
        if value && !can_write {
            bail!("hsd write permission required")
        }
        core_ops::document::set_sync(&registry, value);
        Ok(())
    }

    async fn public(&mut self, self_: Resource<Document>) -> wasmtime::Result<bool> {
        Ok(self.table.get(&self_)?.is_public)
    }

    async fn set_public(&mut self, self_: Resource<Document>, value: bool) -> wasmtime::Result<()> {
        let (lor_doc, id, can_write) = {
            let d = self.table.get(&self_)?;
            (d.lor_doc.as_ref().map(Arc::clone), d.id, d.can_write)
        };
        if !can_write {
            bail!("hsd write permission required")
        }
        let Some(lor_doc) = lor_doc else {
            bail!("set-public requires ownership of the document")
        };
        let Some(actor) = self.actor.clone() else {
            bail!("set-public requires a WDS actor")
        };

        let from = lor_doc.oplog_vv();
        let mut acl = Acl::load(&lor_doc).map_err(|e| wasmtime::Error::msg(e.to_string()))?;
        acl.public = value;
        acl.save(&lor_doc)
            .map_err(|e| wasmtime::Error::msg(e.to_string()))?;
        actor
            .update_record(id, &lor_doc, from)
            .await
            .map_err(|e| wasmtime::Error::msg(e.to_string()))?;

        self.table.get_mut(&self_)?.is_public = value;
        Ok(())
    }

    async fn assets(
        &mut self,
        self_: Resource<Document>,
    ) -> wasmtime::Result<Vec<(String, Vec<u8>)>> {
        use loro::LoroValue;
        let d = self.table.get(&self_)?;
        if !d.can_read {
            bail!("hsd read permission required")
        }
        let Some(ref lor_doc) = d.lor_doc else {
            return Ok(Vec::new());
        };
        let hsd_map = lor_doc.get_map("hsd");
        let value = hsd_map.get_deep_value();
        let LoroValue::Map(root) = &value else {
            return Ok(Vec::new());
        };
        let Some(LoroValue::Map(assets)) = root.get("assets") else {
            return Ok(Vec::new());
        };
        let result = assets
            .iter()
            .filter_map(|(k, v)| {
                if let LoroValue::Binary(bytes) = v {
                    Some((k.clone(), bytes.to_vec()))
                } else {
                    None
                }
            })
            .collect();
        Ok(result)
    }

    async fn add_asset(
        &mut self,
        self_: Resource<Document>,
        name: String,
        blob_id: Vec<u8>,
    ) -> wasmtime::Result<()> {
        let d = self.table.get(&self_)?;
        if !d.can_write {
            bail!("hsd write permission required")
        }
        let Some(ref lor_doc) = d.lor_doc else {
            bail!("add-asset requires document ownership")
        };
        let hsd_map = lor_doc.get_map("hsd");
        let assets = hsd_map
            .get_or_create_container("assets", loro::LoroMap::new())
            .map_err(|e| wasmtime::Error::msg(e.to_string()))?;
        assets
            .insert(name.as_str(), loro::LoroValue::Binary(blob_id.into()))
            .map_err(|e| wasmtime::Error::msg(e.to_string()))?;
        Ok(())
    }

    async fn remove_asset(
        &mut self,
        self_: Resource<Document>,
        name: String,
    ) -> wasmtime::Result<()> {
        let d = self.table.get(&self_)?;
        if !d.can_write {
            bail!("hsd write permission required")
        }
        let Some(ref lor_doc) = d.lor_doc else {
            bail!("remove-asset requires document ownership")
        };
        let hsd_map = lor_doc.get_map("hsd");
        let assets = hsd_map
            .get_or_create_container("assets", loro::LoroMap::new())
            .map_err(|e| wasmtime::Error::msg(e.to_string()))?;
        assets
            .delete(name.as_str())
            .map_err(|e| wasmtime::Error::msg(e.to_string()))?;
        Ok(())
    }

    async fn id(&mut self, self_: Resource<Document>) -> wasmtime::Result<Vec<u8>> {
        Ok(self.table.get(&self_)?.id.as_bytes().to_vec())
    }

    async fn translation(&mut self, self_: Resource<Document>) -> wasmtime::Result<Vec3> {
        let registry = Arc::clone(&self.table.get(&self_)?.registry);
        let t = registry
            .doc_transform
            .lock()
            .expect("doc_transform lock")
            .translation;
        Ok(Vec3 {
            x: t.x,
            y: t.y,
            z: t.z,
        })
    }

    async fn set_translation(
        &mut self,
        self_: Resource<Document>,
        value: Vec3,
    ) -> wasmtime::Result<()> {
        let (doc_id, registry, can_write) = {
            let d = self.table.get(&self_)?;
            (d.id, Arc::clone(&d.registry), d.can_write)
        };
        if !can_write {
            bail!("hsd write permission required")
        }
        let transform = {
            let mut t = registry.doc_transform.lock().expect("doc_transform lock");
            t.translation = bevy::math::Vec3::new(value.x, value.y, value.z);
            *t
        };
        self.command_queue.lock().expect("cmd queue lock").push(
            move |world: &mut bevy::prelude::World| {
                world.trigger(HsdDocTransformSet { doc_id, transform });
            },
        );
        Ok(())
    }

    async fn rotation(&mut self, self_: Resource<Document>) -> wasmtime::Result<Quat> {
        let registry = Arc::clone(&self.table.get(&self_)?.registry);
        let r = registry
            .doc_transform
            .lock()
            .expect("doc_transform lock")
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
        self_: Resource<Document>,
        value: Quat,
    ) -> wasmtime::Result<()> {
        let (doc_id, registry, can_write) = {
            let d = self.table.get(&self_)?;
            (d.id, Arc::clone(&d.registry), d.can_write)
        };
        if !can_write {
            bail!("hsd write permission required")
        }
        let transform = {
            let mut t = registry.doc_transform.lock().expect("doc_transform lock");
            t.rotation =
                bevy::math::Quat::from_xyzw(value.x, value.y, value.z, value.w).normalize();
            *t
        };
        self.command_queue.lock().expect("cmd queue lock").push(
            move |world: &mut bevy::prelude::World| {
                world.trigger(HsdDocTransformSet { doc_id, transform });
            },
        );
        Ok(())
    }

    async fn scale(&mut self, self_: Resource<Document>) -> wasmtime::Result<Vec3> {
        let registry = Arc::clone(&self.table.get(&self_)?.registry);
        let s = registry
            .doc_transform
            .lock()
            .expect("doc_transform lock")
            .scale;
        Ok(Vec3 {
            x: s.x,
            y: s.y,
            z: s.z,
        })
    }

    async fn set_scale(&mut self, self_: Resource<Document>, value: Vec3) -> wasmtime::Result<()> {
        let (doc_id, registry, can_write) = {
            let d = self.table.get(&self_)?;
            (d.id, Arc::clone(&d.registry), d.can_write)
        };
        if !can_write {
            bail!("hsd write permission required")
        }
        let transform = {
            let mut t = registry.doc_transform.lock().expect("doc_transform lock");
            t.scale = bevy::math::Vec3::new(value.x, value.y, value.z);
            *t
        };
        self.command_queue.lock().expect("cmd queue lock").push(
            move |world: &mut bevy::prelude::World| {
                world.trigger(HsdDocTransformSet { doc_id, transform });
            },
        );
        Ok(())
    }

    async fn transform(&mut self, self_: Resource<Document>) -> wasmtime::Result<Transform> {
        let registry = Arc::clone(&self.table.get(&self_)?.registry);
        let bt = *registry.doc_transform.lock().expect("doc_transform lock");
        Ok(Transform {
            translation: Vec3 {
                x: bt.translation.x,
                y: bt.translation.y,
                z: bt.translation.z,
            },
            rotation: Quat {
                x: bt.rotation.x,
                y: bt.rotation.y,
                z: bt.rotation.z,
                w: bt.rotation.w,
            },
            scale: Vec3 {
                x: bt.scale.x,
                y: bt.scale.y,
                z: bt.scale.z,
            },
        })
    }

    async fn set_transform(
        &mut self,
        self_: Resource<Document>,
        value: Transform,
    ) -> wasmtime::Result<()> {
        let (doc_id, registry, can_write) = {
            let d = self.table.get(&self_)?;
            (d.id, Arc::clone(&d.registry), d.can_write)
        };
        if !can_write {
            bail!("hsd write permission required")
        }
        let transform = BevyTransform {
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
        *registry.doc_transform.lock().expect("doc_transform lock") = transform;
        self.command_queue.lock().expect("cmd queue lock").push(
            move |world: &mut bevy::prelude::World| {
                world.trigger(HsdDocTransformSet { doc_id, transform });
            },
        );
        Ok(())
    }

    async fn global_transform(&mut self, self_: Resource<Document>) -> wasmtime::Result<Transform> {
        // HsdDoc is always a root entity; world transform equals local transform.
        self.transform(self_).await
    }

    async fn drop(&mut self, rep: Resource<Document>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
