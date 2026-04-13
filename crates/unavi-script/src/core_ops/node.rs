use std::sync::Arc;
use std::sync::atomic::Ordering;

use bevy::prelude::{Transform, World};
use bevy_hsd::cache::NodeInner;
use bevy_hsd::hydrate::compile::node::{
    HsdNodeMaterialSet, HsdNodeMeshSet, HsdNodeNameSet, HsdNodeParentSet, HsdNodeTransformSet,
};
use bevy_hsd::hydrate::events::{NodeRef, ScriptCommandQueue};
use smol_str::SmolStr;

pub fn set_name(
    inner: &NodeInner,
    doc_id: blake3::Hash,
    value: Option<String>,
    cmds: &mut ScriptCommandQueue,
) {
    if inner.is_virtual {
        return;
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
        let id = inner.id.clone();
        cmds.push(move |world: &mut World| {
            world.trigger(HsdNodeNameSet {
                doc_id,
                id,
                name: value,
            });
        });
    }
}

pub fn set_translation(
    inner: &NodeInner,
    doc_id: blake3::Hash,
    x: f32,
    y: f32,
    z: f32,
    cmds: &mut ScriptCommandQueue,
) {
    if inner.is_virtual {
        return;
    }
    inner
        .state
        .lock()
        .expect("node state lock")
        .transform
        .translation = bevy::math::Vec3::new(x, y, z);
    if inner.sync.load(Ordering::Relaxed) {
        inner
            .hsd_changes
            .lock()
            .expect("hsd_changes lock")
            .translation = Some([f64::from(x), f64::from(y), f64::from(z)]);
    } else {
        let id = inner.id.clone();
        let transform = inner.state.lock().expect("node state lock").transform;
        cmds.push(move |world: &mut World| {
            world.trigger(HsdNodeTransformSet {
                doc_id,
                id,
                transform,
            });
        });
    }
}

pub fn set_rotation(
    inner: &NodeInner,
    doc_id: blake3::Hash,
    x: f32,
    y: f32,
    z: f32,
    w: f32,
    cmds: &mut ScriptCommandQueue,
) {
    if inner.is_virtual {
        return;
    }
    inner
        .state
        .lock()
        .expect("node state lock")
        .transform
        .rotation = bevy::math::Quat::from_xyzw(x, y, z, w);
    if inner.sync.load(Ordering::Relaxed) {
        inner.hsd_changes.lock().expect("hsd_changes lock").rotation =
            Some([f64::from(x), f64::from(y), f64::from(z), f64::from(w)]);
    } else {
        let id = inner.id.clone();
        let transform = inner.state.lock().expect("node state lock").transform;
        cmds.push(move |world: &mut World| {
            world.trigger(HsdNodeTransformSet {
                doc_id,
                id,
                transform,
            });
        });
    }
}

pub fn set_scale(
    inner: &NodeInner,
    doc_id: blake3::Hash,
    x: f32,
    y: f32,
    z: f32,
    cmds: &mut ScriptCommandQueue,
) {
    if inner.is_virtual {
        return;
    }
    inner.state.lock().expect("node state lock").transform.scale = bevy::math::Vec3::new(x, y, z);
    if inner.sync.load(Ordering::Relaxed) {
        inner.hsd_changes.lock().expect("hsd_changes lock").scale =
            Some([f64::from(x), f64::from(y), f64::from(z)]);
    } else {
        let id = inner.id.clone();
        let transform = inner.state.lock().expect("node state lock").transform;
        cmds.push(move |world: &mut World| {
            world.trigger(HsdNodeTransformSet {
                doc_id,
                id,
                transform,
            });
        });
    }
}

pub fn set_transform(
    inner: &NodeInner,
    doc_id: blake3::Hash,
    new_transform: Transform,
    cmds: &mut ScriptCommandQueue,
) {
    if inner.is_virtual {
        return;
    }
    inner.state.lock().expect("node state lock").transform = new_transform;
    if inner.sync.load(Ordering::Relaxed) {
        let t = new_transform.translation;
        let r = new_transform.rotation;
        let s = new_transform.scale;
        let mut ch = inner.hsd_changes.lock().expect("hsd_changes lock");
        ch.translation = Some([f64::from(t.x), f64::from(t.y), f64::from(t.z)]);
        ch.rotation = Some([
            f64::from(r.x),
            f64::from(r.y),
            f64::from(r.z),
            f64::from(r.w),
        ]);
        ch.scale = Some([f64::from(s.x), f64::from(s.y), f64::from(s.z)]);
    } else {
        let id = inner.id.clone();
        cmds.push(move |world: &mut World| {
            world.trigger(HsdNodeTransformSet {
                doc_id,
                id,
                transform: new_transform,
            });
        });
    }
}

pub fn set_mesh(
    inner: &NodeInner,
    doc_id: blake3::Hash,
    mesh_id: Option<SmolStr>,
    cmds: &mut ScriptCommandQueue,
) {
    if inner.is_virtual {
        return;
    }
    inner
        .state
        .lock()
        .expect("node state lock")
        .mesh
        .clone_from(&mesh_id);
    if inner.sync.load(Ordering::Relaxed) {
        inner.hsd_changes.lock().expect("hsd_changes lock").mesh = Some(mesh_id);
    } else {
        let id = inner.id.clone();
        cmds.push(move |world: &mut World| {
            world.trigger(HsdNodeMeshSet {
                doc_id,
                id,
                mesh: mesh_id,
            });
        });
    }
}

pub fn set_material(
    inner: &NodeInner,
    doc_id: blake3::Hash,
    mat_id: Option<SmolStr>,
    cmds: &mut ScriptCommandQueue,
) {
    if inner.is_virtual {
        return;
    }
    inner
        .state
        .lock()
        .expect("node state lock")
        .material
        .clone_from(&mat_id);
    if inner.sync.load(Ordering::Relaxed) {
        inner.hsd_changes.lock().expect("hsd_changes lock").material = Some(mat_id);
    } else {
        let id = inner.id.clone();
        cmds.push(move |world: &mut World| {
            world.trigger(HsdNodeMaterialSet {
                doc_id,
                id,
                material: mat_id,
            });
        });
    }
}

pub fn add_child(
    parent_inner: &Arc<NodeInner>,
    child_inner: &Arc<NodeInner>,
    doc_id: blake3::Hash,
    cmds: &mut ScriptCommandQueue,
) {
    {
        let mut parent_state = parent_inner.state.lock().expect("parent state lock");
        if !parent_state.children.iter().any(|c| c.id == child_inner.id) {
            parent_state.children.push(Arc::clone(child_inner));
        }
    }
    {
        child_inner.state.lock().expect("child state lock").parent =
            Some(Arc::downgrade(parent_inner));
    }

    let parent_ent = *parent_inner.entity.lock().expect("entity lock");
    let child_ent = *child_inner.entity.lock().expect("entity lock");
    let child_ref = child_ent.map_or_else(|| NodeRef::Id(child_inner.id.clone()), NodeRef::Entity);
    let parent_ref =
        parent_ent.map_or_else(|| NodeRef::Id(parent_inner.id.clone()), NodeRef::Entity);
    cmds.push(move |world: &mut World| {
        world.trigger(HsdNodeParentSet {
            doc_id,
            child: child_ref,
            parent: Some(parent_ref),
        });
    });
}

pub fn remove_child(
    child_inner: &Arc<NodeInner>,
    doc_id: blake3::Hash,
    cmds: &mut ScriptCommandQueue,
) {
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

    let child_ent = *child_inner.entity.lock().expect("entity lock");
    let child_ref = child_ent.map_or_else(|| NodeRef::Id(child_inner.id.clone()), NodeRef::Entity);
    cmds.push(move |world: &mut World| {
        world.trigger(HsdNodeParentSet {
            doc_id,
            child: child_ref,
            parent: None,
        });
    });
}
