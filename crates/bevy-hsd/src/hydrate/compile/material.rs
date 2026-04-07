use bevy::prelude::*;
use bevy_wds::{BlobDeps, BlobDepsLoaded, BlobResponse};
use smol_str::SmolStr;

use crate::{CompiledMaterial, HsdChild, cache::SceneRegistry, data::HsdMaterial};

#[derive(Event)]
pub struct HsdMaterialAlphaCutoffSet {
    pub doc: Entity,
    pub id: SmolStr,
    pub value: f32,
}

#[derive(Event)]
pub struct HsdMaterialAlphaModeSet {
    pub doc: Entity,
    pub id: SmolStr,
    pub mode: Option<String>,
}

#[derive(Event)]
pub struct HsdMaterialBaseColorSet {
    pub doc: Entity,
    pub id: SmolStr,
    pub color: [f32; 4],
}

#[derive(Event)]
pub struct HsdMaterialDespawned {
    pub doc: Entity,
    pub id: SmolStr,
}

#[derive(Event)]
pub struct HsdMaterialDoubleSidedSet {
    pub doc: Entity,
    pub id: SmolStr,
    pub value: bool,
}

#[derive(Event)]
pub struct HsdMaterialMetallicSet {
    pub doc: Entity,
    pub id: SmolStr,
    pub value: f32,
}

#[derive(Event)]
pub struct HsdMaterialNameSet {
    pub doc: Entity,
    pub id: SmolStr,
    pub name: Option<String>,
}

#[derive(Event)]
pub struct HsdMaterialRoughnessSet {
    pub doc: Entity,
    pub id: SmolStr,
    pub value: f32,
}

#[derive(Event)]
pub struct HsdMaterialSpawned {
    pub doc: Entity,
    pub id: SmolStr,
    pub initial: Option<HsdMaterial>,
}

#[derive(Event)]
pub struct HsdMaterialUnlitSet {
    pub doc: Entity,
    pub id: SmolStr,
    pub value: bool,
}

#[derive(Component, Default)]
#[require(BlobDeps)]
pub struct MaterialParams {
    pub alpha_cutoff: Option<f32>,
    pub alpha_mode: Option<String>,
    pub base_color: Option<Color>,
    pub double_sided: Option<bool>,
    pub metallic: Option<f32>,
    pub roughness: Option<f32>,
    pub unlit: Option<bool>,
    pub base_color_texture: Option<Entity>,
    _metallic_roughness_texture: Option<Entity>,
    _normal_texture: Option<Entity>,
    _occlusion_texture: Option<Entity>,
}

fn material_params_from_hsd(hsd: &HsdMaterial) -> MaterialParams {
    let mut params = MaterialParams::default();
    if let Some(color) = &hsd.base_color
        && color.len() >= 3
    {
        let [r, g, b] = [color[0] as f32, color[1] as f32, color[2] as f32];
        let a = color.get(3).copied().unwrap_or(1.0) as f32;
        params.base_color = Some(Color::srgba(r, g, b, a));
    }
    params.alpha_cutoff = hsd.alpha_cutoff.map(|v| v as f32);
    params.alpha_mode = hsd.alpha_mode.as_ref().map(ToString::to_string);
    params.double_sided = hsd.double_sided;
    params.metallic = hsd.metallic.map(|v| v as f32);
    params.roughness = hsd.roughness.map(|v| v as f32);
    params.unlit = hsd.unlit;
    params
}

pub(crate) fn handle_hsd_material_spawned(
    trigger: On<HsdMaterialSpawned>,
    registries: Query<&SceneRegistry>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "material spawned");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let inner = registry
        .0
        .materials
        .lock()
        .expect("materials lock")
        .get(&ev.id)
        .cloned();
    let Some(inner) = inner else { return };
    if inner.entity.lock().expect("entity lock").is_some() {
        return;
    }
    let params = ev
        .initial
        .as_ref()
        .map(material_params_from_hsd)
        .unwrap_or_default();
    let ent = commands.spawn((HsdChild { doc: ev.doc }, params)).id();
    *inner.entity.lock().expect("entity lock") = Some(ent);
}

pub(crate) fn handle_hsd_material_despawned(
    trigger: On<HsdMaterialDespawned>,
    registries: Query<&SceneRegistry>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "material despawned");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let inner = {
        let mut mats = registry.0.materials.lock().expect("materials lock");
        mats.remove(&ev.id)
    };
    let Some(inner) = inner else { return };
    if let Some(ent) = *inner.entity.lock().expect("entity lock")
        && let Ok(mut ent) = commands.get_entity(ent)
    {
        ent.despawn();
    }
}

pub(crate) fn handle_hsd_material_alpha_cutoff_set(
    trigger: On<HsdMaterialAlphaCutoffSet>,
    registries: Query<&SceneRegistry>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, value = ev.value, "material alpha cutoff set");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let ent = registry
        .0
        .materials
        .lock()
        .expect("materials lock")
        .get(&ev.id)
        .and_then(|m| *m.entity.lock().expect("entity lock"));
    let Some(ent) = ent else { return };
    if let Ok(mut p) = params.get_mut(ent) {
        p.alpha_cutoff = Some(ev.value);
    }
}

pub(crate) fn handle_hsd_material_alpha_mode_set(
    trigger: On<HsdMaterialAlphaModeSet>,
    registries: Query<&SceneRegistry>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, mode = ?ev.mode, "material alpha mode set");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let ent = registry
        .0
        .materials
        .lock()
        .expect("materials lock")
        .get(&ev.id)
        .and_then(|m| *m.entity.lock().expect("entity lock"));
    let Some(ent) = ent else { return };
    if let Ok(mut p) = params.get_mut(ent) {
        p.alpha_mode.clone_from(&ev.mode);
    }
}

pub(crate) fn handle_hsd_material_base_color_set(
    trigger: On<HsdMaterialBaseColorSet>,
    registries: Query<&SceneRegistry>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, color = ?ev.color, "material base color set");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let ent = registry
        .0
        .materials
        .lock()
        .expect("materials lock")
        .get(&ev.id)
        .and_then(|m| *m.entity.lock().expect("entity lock"));
    let Some(ent) = ent else { return };
    if let Ok(mut p) = params.get_mut(ent) {
        let [r, g, b, a] = ev.color;
        p.base_color = Some(Color::srgba(r, g, b, a));
    }
}

pub(crate) fn handle_hsd_material_double_sided_set(
    trigger: On<HsdMaterialDoubleSidedSet>,
    registries: Query<&SceneRegistry>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, value = ev.value, "material double sided set");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let ent = registry
        .0
        .materials
        .lock()
        .expect("materials lock")
        .get(&ev.id)
        .and_then(|m| *m.entity.lock().expect("entity lock"));
    let Some(ent) = ent else { return };
    if let Ok(mut p) = params.get_mut(ent) {
        p.double_sided = Some(ev.value);
    }
}

pub(crate) fn handle_hsd_material_metallic_set(
    trigger: On<HsdMaterialMetallicSet>,
    registries: Query<&SceneRegistry>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, value = ev.value, "material metallic set");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let ent = registry
        .0
        .materials
        .lock()
        .expect("materials lock")
        .get(&ev.id)
        .and_then(|m| *m.entity.lock().expect("entity lock"));
    let Some(ent) = ent else { return };
    if let Ok(mut p) = params.get_mut(ent) {
        p.metallic = Some(ev.value);
    }
}

pub(crate) fn handle_hsd_material_name_set(
    trigger: On<HsdMaterialNameSet>,
    registries: Query<&SceneRegistry>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, name = ?ev.name, "material name set");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let ent = registry
        .0
        .materials
        .lock()
        .expect("materials lock")
        .get(&ev.id)
        .and_then(|m| *m.entity.lock().expect("entity lock"));
    let Some(ent) = ent else { return };
    let Ok(mut entity_cmd) = commands.get_entity(ent) else {
        return;
    };
    if let Some(ref name) = ev.name {
        entity_cmd.insert(Name::new(name.clone()));
    } else {
        entity_cmd.remove::<Name>();
    }
}

pub(crate) fn handle_hsd_material_unlit_set(
    trigger: On<HsdMaterialUnlitSet>,
    registries: Query<&SceneRegistry>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, value = ev.value, "material unlit set");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let ent = registry
        .0
        .materials
        .lock()
        .expect("materials lock")
        .get(&ev.id)
        .and_then(|m| *m.entity.lock().expect("entity lock"));
    let Some(ent) = ent else { return };
    if let Ok(mut p) = params.get_mut(ent) {
        p.unlit = Some(ev.value);
    }
}

pub(crate) fn handle_hsd_material_roughness_set(
    trigger: On<HsdMaterialRoughnessSet>,
    registries: Query<&SceneRegistry>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, value = ev.value, "material roughness set");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let ent = registry
        .0
        .materials
        .lock()
        .expect("materials lock")
        .get(&ev.id)
        .and_then(|m| *m.entity.lock().expect("entity lock"));
    let Some(ent) = ent else { return };
    if let Ok(mut p) = params.get_mut(ent) {
        p.roughness = Some(ev.value);
    }
}

fn build_standard_material(params: &MaterialParams) -> StandardMaterial {
    let mut material = StandardMaterial::default();
    if let Some(value) = params.base_color {
        material.base_color = value;
    }
    material.alpha_mode = match params.alpha_mode.as_deref() {
        Some("add") => AlphaMode::Add,
        Some("blend") => AlphaMode::Blend,
        Some("mask") => AlphaMode::Mask(params.alpha_cutoff.unwrap_or(0.5)),
        Some("multiply") => AlphaMode::Multiply,
        Some("opaque") => AlphaMode::Opaque,
        Some("premultiplied") => AlphaMode::Premultiplied,
        _ => {
            if material.base_color.alpha() < 1.0 {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            }
        }
    };
    if let Some(value) = params.double_sided {
        material.double_sided = value;
    }
    if let Some(value) = params.metallic {
        material.metallic = value;
    }
    if let Some(value) = params.roughness {
        material.perceptual_roughness = value;
    }
    if let Some(value) = params.unlit {
        material.unlit = value;
    }
    material
}

pub(crate) fn on_material_blobs_loaded(
    trigger: On<Add, BlobDepsLoaded>,
    mat_params: Query<(&MaterialParams, Option<&CompiledMaterial>)>,
    mut mat_assets: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut blobs: Query<&mut BlobResponse>,
) {
    let ent = trigger.entity;
    let Ok((params, existing)) = mat_params.get(ent) else {
        return;
    };

    if let Some(value) = params.base_color_texture {
        let Ok(Some(_bytes)) = blobs.get_mut(value).map(|mut b| b.0.take()) else {
            return;
        };
        // TODO: load image details from HSD
    }

    let material = build_standard_material(params);
    debug!("compiled material {ent}");
    if let Some(CompiledMaterial(handle)) = existing {
        if let Some(asset) = mat_assets.get_mut(handle) {
            *asset = material;
            commands
                .entity(ent)
                .remove::<BlobDeps>()
                .remove::<BlobDepsLoaded>();
        }
    } else {
        let handle = mat_assets.add(material);
        commands
            .entity(ent)
            .insert(CompiledMaterial(handle))
            .remove::<BlobDeps>()
            .remove::<BlobDepsLoaded>();
    }
}

pub(crate) fn recompile_changed_materials(
    changed: Query<(&MaterialParams, &CompiledMaterial), Changed<MaterialParams>>,
    mut mat_assets: ResMut<Assets<StandardMaterial>>,
) {
    for (params, compiled) in &changed {
        let Some(asset) = mat_assets.get_mut(&compiled.0) else {
            continue;
        };
        *asset = build_standard_material(params);
    }
}
