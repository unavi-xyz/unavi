use std::sync::atomic::Ordering;

use bevy::prelude::World;
use bevy_hsd::cache::MaterialInner;
use bevy_hsd::hydrate::compile::material::{
    HsdMaterialAlphaCutoffSet, HsdMaterialAlphaModeSet, HsdMaterialBaseColorSet,
    HsdMaterialDoubleSidedSet, HsdMaterialMetallicSet, HsdMaterialNameSet, HsdMaterialRoughnessSet,
    HsdMaterialUnlitSet,
};
use bevy_hsd::hydrate::events::ScriptCommandQueue;

pub fn set_name(
    inner: &MaterialInner,
    doc_id: blake3::Hash,
    value: Option<String>,
    cmds: &mut ScriptCommandQueue,
) {
    inner
        .state
        .lock()
        .expect("material state lock")
        .name
        .clone_from(&value);
    if inner.sync.load(Ordering::Relaxed) {
        inner.hsd_changes.lock().expect("hsd_changes lock").name = Some(value);
    } else {
        let id = inner.id.clone();
        cmds.push(move |world: &mut World| {
            world.trigger(HsdMaterialNameSet {
                doc_id,
                id,
                name: value,
            });
        });
    }
}

pub fn set_base_color(
    inner: &MaterialInner,
    doc_id: blake3::Hash,
    color: [f32; 4],
    cmds: &mut ScriptCommandQueue,
) {
    inner.state.lock().expect("material state lock").base_color = color;
    if inner.sync.load(Ordering::Relaxed) {
        inner
            .hsd_changes
            .lock()
            .expect("hsd_changes lock")
            .base_color = Some(color.map(f64::from));
    } else {
        let id = inner.id.clone();
        cmds.push(move |world: &mut World| {
            world.trigger(HsdMaterialBaseColorSet { doc_id, id, color });
        });
    }
}

pub fn set_metallic(
    inner: &MaterialInner,
    doc_id: blake3::Hash,
    value: f32,
    cmds: &mut ScriptCommandQueue,
) {
    inner.state.lock().expect("material state lock").metallic = value;
    if inner.sync.load(Ordering::Relaxed) {
        inner.hsd_changes.lock().expect("hsd_changes lock").metallic = Some(f64::from(value));
    } else {
        let id = inner.id.clone();
        cmds.push(move |world: &mut World| {
            world.trigger(HsdMaterialMetallicSet { doc_id, id, value });
        });
    }
}

pub fn set_roughness(
    inner: &MaterialInner,
    doc_id: blake3::Hash,
    value: f32,
    cmds: &mut ScriptCommandQueue,
) {
    inner.state.lock().expect("material state lock").roughness = value;
    if inner.sync.load(Ordering::Relaxed) {
        inner
            .hsd_changes
            .lock()
            .expect("hsd_changes lock")
            .roughness = Some(f64::from(value));
    } else {
        let id = inner.id.clone();
        cmds.push(move |world: &mut World| {
            world.trigger(HsdMaterialRoughnessSet { doc_id, id, value });
        });
    }
}

pub fn set_double_sided(
    inner: &MaterialInner,
    doc_id: blake3::Hash,
    value: bool,
    cmds: &mut ScriptCommandQueue,
) {
    inner
        .state
        .lock()
        .expect("material state lock")
        .double_sided = value;
    if inner.sync.load(Ordering::Relaxed) {
        inner
            .hsd_changes
            .lock()
            .expect("hsd_changes lock")
            .double_sided = Some(value);
    } else {
        let id = inner.id.clone();
        cmds.push(move |world: &mut World| {
            world.trigger(HsdMaterialDoubleSidedSet { doc_id, id, value });
        });
    }
}

pub fn set_unlit(
    inner: &MaterialInner,
    doc_id: blake3::Hash,
    value: bool,
    cmds: &mut ScriptCommandQueue,
) {
    inner.state.lock().expect("material state lock").unlit = value;
    if inner.sync.load(Ordering::Relaxed) {
        inner.hsd_changes.lock().expect("hsd_changes lock").unlit = Some(value);
    } else {
        let id = inner.id.clone();
        cmds.push(move |world: &mut World| {
            world.trigger(HsdMaterialUnlitSet { doc_id, id, value });
        });
    }
}

pub fn set_alpha_cutoff(
    inner: &MaterialInner,
    doc_id: blake3::Hash,
    value: f32,
    cmds: &mut ScriptCommandQueue,
) {
    inner
        .state
        .lock()
        .expect("material state lock")
        .alpha_cutoff = Some(value);
    if inner.sync.load(Ordering::Relaxed) {
        inner
            .hsd_changes
            .lock()
            .expect("hsd_changes lock")
            .alpha_cutoff = Some(f64::from(value));
    } else {
        let id = inner.id.clone();
        cmds.push(move |world: &mut World| {
            world.trigger(HsdMaterialAlphaCutoffSet { doc_id, id, value });
        });
    }
}

pub fn set_alpha_mode(
    inner: &MaterialInner,
    doc_id: blake3::Hash,
    mode_str: Option<String>,
    cmds: &mut ScriptCommandQueue,
) {
    inner
        .state
        .lock()
        .expect("material state lock")
        .alpha_mode
        .clone_from(&mode_str);
    if inner.sync.load(Ordering::Relaxed) {
        inner
            .hsd_changes
            .lock()
            .expect("hsd_changes lock")
            .alpha_mode = Some(mode_str);
    } else {
        let id = inner.id.clone();
        cmds.push(move |world: &mut World| {
            world.trigger(HsdMaterialAlphaModeSet {
                doc_id,
                id,
                mode: mode_str,
            });
        });
    }
}
