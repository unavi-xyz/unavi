use std::sync::atomic::Ordering;

use bevy::mesh::PrimitiveTopology;
use bevy::prelude::{Entity, World};
use bevy_hsd::cache::MeshInner;
use bevy_hsd::hydrate::compile::mesh::{HsdMeshGeometrySet, MeshGeometrySource};
use bevy_hsd::hydrate::events::ScriptCommandQueue;

pub fn set_name(inner: &MeshInner, value: Option<String>) {
    inner
        .state
        .lock()
        .expect("mesh state lock")
        .name
        .clone_from(&value);
    if inner.sync.load(Ordering::Relaxed) {
        inner.hsd_changes.lock().expect("hsd_changes lock").name = Some(value);
    }
}

pub fn set_topology(inner: &MeshInner, topo: PrimitiveTopology) {
    inner.state.lock().expect("mesh state lock").topology = topo;
    if inner.sync.load(Ordering::Relaxed) {
        inner.hsd_changes.lock().expect("hsd_changes lock").topology = Some(bevy_topo_to_i64(topo));
    }
}

pub fn set_indices(
    inner: &MeshInner,
    doc: Entity,
    indices: Option<Vec<u32>>,
    cmds: &mut ScriptCommandQueue,
) {
    inner.state.lock().expect("mesh state lock").indices = indices;
    push_geometry_set(inner, doc, cmds);
}

pub fn set_positions(
    inner: &MeshInner,
    doc: Entity,
    values: Option<Vec<f32>>,
    cmds: &mut ScriptCommandQueue,
) {
    inner.state.lock().expect("mesh state lock").positions = values;
    push_geometry_set(inner, doc, cmds);
}

pub fn set_normals(
    inner: &MeshInner,
    doc: Entity,
    values: Option<Vec<f32>>,
    cmds: &mut ScriptCommandQueue,
) {
    inner.state.lock().expect("mesh state lock").normals = values;
    push_geometry_set(inner, doc, cmds);
}

pub fn set_tangents(
    inner: &MeshInner,
    doc: Entity,
    values: Option<Vec<f32>>,
    cmds: &mut ScriptCommandQueue,
) {
    inner.state.lock().expect("mesh state lock").tangents = values;
    push_geometry_set(inner, doc, cmds);
}

pub fn set_colors(
    inner: &MeshInner,
    doc: Entity,
    values: Option<Vec<f32>>,
    cmds: &mut ScriptCommandQueue,
) {
    inner.state.lock().expect("mesh state lock").colors = values;
    push_geometry_set(inner, doc, cmds);
}

pub fn set_uv0(
    inner: &MeshInner,
    doc: Entity,
    values: Option<Vec<f32>>,
    cmds: &mut ScriptCommandQueue,
) {
    inner.state.lock().expect("mesh state lock").uv0 = values;
    push_geometry_set(inner, doc, cmds);
}

pub fn set_uv1(
    inner: &MeshInner,
    doc: Entity,
    values: Option<Vec<f32>>,
    cmds: &mut ScriptCommandQueue,
) {
    inner.state.lock().expect("mesh state lock").uv1 = values;
    push_geometry_set(inner, doc, cmds);
}

fn push_geometry_set(inner: &MeshInner, doc: Entity, cmds: &mut ScriptCommandQueue) {
    let id = inner.id.clone();
    cmds.push(move |world: &mut World| {
        world.trigger(HsdMeshGeometrySet {
            doc,
            id,
            source: MeshGeometrySource::Inline,
        });
    });
}

#[must_use]
pub const fn bevy_topo_to_i64(t: PrimitiveTopology) -> i64 {
    match t {
        PrimitiveTopology::PointList => 0,
        PrimitiveTopology::LineList => 1,
        PrimitiveTopology::LineStrip => 2,
        PrimitiveTopology::TriangleList => 3,
        PrimitiveTopology::TriangleStrip => 4,
    }
}
