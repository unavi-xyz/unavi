use bevy::prelude::*;

use crate::cache::{MeshDirty, SceneRegistry};

use super::compile::material::{
    HsdMaterialAlphaCutoffSet, HsdMaterialAlphaModeSet, HsdMaterialBaseColorSet,
    HsdMaterialDespawned, HsdMaterialDoubleSidedSet, HsdMaterialMetallicSet, HsdMaterialNameSet,
    HsdMaterialRoughnessSet, HsdMaterialSpawned,
};
use super::compile::mesh::{
    HsdMeshDespawned, HsdMeshGeometrySet, HsdMeshSpawned, MeshGeometrySource,
};
use super::compile::node::{
    HsdNodeColliderSet, HsdNodeDespawned, HsdNodeMaterialSet, HsdNodeMeshSet, HsdNodeNameSet,
    HsdNodeParentSet, HsdNodeRigidBodySet, HsdNodeSpawned, HsdNodeTransformSet,
};
use super::events::{ScriptEventQueue, ScriptQueuedEvent};

#[expect(clippy::too_many_lines)]
pub(crate) fn flush_script_dirty(
    docs: Query<(Entity, &SceneRegistry, &ScriptEventQueue)>,
    mut commands: Commands,
) {
    for (doc_ent, registry, event_queue) in &docs {
        let queued: Vec<ScriptQueuedEvent> = event_queue
            .0
            .lock()
            .expect("script event queue lock")
            .drain(..)
            .collect();
        for ev in queued {
            match ev {
                ScriptQueuedEvent::NodeSpawned { id } => {
                    commands.trigger(HsdNodeSpawned { doc: doc_ent, id });
                }
                ScriptQueuedEvent::NodeDespawned { id } => {
                    commands.trigger(HsdNodeDespawned { doc: doc_ent, id });
                }
                ScriptQueuedEvent::NodeParentSet { id, parent } => {
                    commands.trigger(HsdNodeParentSet {
                        doc: doc_ent,
                        id,
                        parent,
                    });
                }
                ScriptQueuedEvent::MeshSpawned { id } => {
                    commands.trigger(HsdMeshSpawned { doc: doc_ent, id });
                }
                ScriptQueuedEvent::MeshDespawned { id } => {
                    commands.trigger(HsdMeshDespawned { doc: doc_ent, id });
                }
                ScriptQueuedEvent::MaterialSpawned { id } => {
                    commands.trigger(HsdMaterialSpawned { doc: doc_ent, id });
                }
                ScriptQueuedEvent::MaterialDespawned { id } => {
                    commands.trigger(HsdMaterialDespawned { doc: doc_ent, id });
                }
            }
        }

        {
            let nodes = registry.0.nodes.lock().expect("nodes lock");
            for inner in nodes.iter() {
                if inner.is_virtual {
                    continue;
                }
                let dirty = {
                    let mut d = inner.dirty.lock().expect("dirty lock");
                    if !d.any() {
                        continue;
                    }
                    std::mem::take(&mut *d)
                };
                let id = inner.id.clone();
                let state = inner.state.lock().expect("node state lock");
                if dirty.transform {
                    commands.trigger(HsdNodeTransformSet {
                        doc: doc_ent,
                        id: id.clone(),
                        transform: state.transform,
                    });
                }
                if dirty.mesh {
                    commands.trigger(HsdNodeMeshSet {
                        doc: doc_ent,
                        id: id.clone(),
                        mesh: state.mesh.clone(),
                    });
                }
                if dirty.material {
                    commands.trigger(HsdNodeMaterialSet {
                        doc: doc_ent,
                        id: id.clone(),
                        material: state.material.clone(),
                    });
                }
                if dirty.name {
                    commands.trigger(HsdNodeNameSet {
                        doc: doc_ent,
                        id: id.clone(),
                        name: state.name.clone(),
                    });
                }
                if dirty.collider {
                    commands.trigger(HsdNodeColliderSet {
                        doc: doc_ent,
                        id: id.clone(),
                        collider: state.collider.clone(),
                    });
                }
                if dirty.rigid_body {
                    commands.trigger(HsdNodeRigidBodySet {
                        doc: doc_ent,
                        id: id.clone(),
                        rigid_body: state.rigid_body.clone(),
                    });
                }
            }
        }

        {
            let meshes = registry.0.meshes.lock().expect("meshes lock");
            for inner in meshes.values() {
                {
                    let mut d = inner.dirty.lock().expect("dirty lock");
                    if !d.any() {
                        continue;
                    }
                    *d = MeshDirty::default();
                }
                commands.trigger(HsdMeshGeometrySet {
                    doc: doc_ent,
                    id: inner.id.clone(),
                    source: MeshGeometrySource::Inline,
                });
            }
        }

        {
            let materials = registry.0.materials.lock().expect("materials lock");
            for inner in materials.values() {
                let dirty = {
                    let mut d = inner.dirty.lock().expect("dirty lock");
                    if !d.any() {
                        continue;
                    }
                    std::mem::take(&mut *d)
                };
                let id = inner.id.clone();
                let state = inner.state.lock().expect("material state lock");
                if dirty.base_color {
                    commands.trigger(HsdMaterialBaseColorSet {
                        doc: doc_ent,
                        id: id.clone(),
                        color: state.base_color,
                    });
                }
                if dirty.metallic {
                    commands.trigger(HsdMaterialMetallicSet {
                        doc: doc_ent,
                        id: id.clone(),
                        value: state.metallic,
                    });
                }
                if dirty.roughness {
                    commands.trigger(HsdMaterialRoughnessSet {
                        doc: doc_ent,
                        id: id.clone(),
                        value: state.roughness,
                    });
                }
                if dirty.alpha_cutoff {
                    commands.trigger(HsdMaterialAlphaCutoffSet {
                        doc: doc_ent,
                        id: id.clone(),
                        value: state.alpha_cutoff.unwrap_or(0.5),
                    });
                }
                if dirty.alpha_mode {
                    commands.trigger(HsdMaterialAlphaModeSet {
                        doc: doc_ent,
                        id: id.clone(),
                        mode: state.alpha_mode.clone(),
                    });
                }
                if dirty.double_sided {
                    commands.trigger(HsdMaterialDoubleSidedSet {
                        doc: doc_ent,
                        id: id.clone(),
                        value: state.double_sided,
                    });
                }
                if dirty.name {
                    commands.trigger(HsdMaterialNameSet {
                        doc: doc_ent,
                        id: id.clone(),
                        name: state.name.clone(),
                    });
                }
            }
        }
    }
}
