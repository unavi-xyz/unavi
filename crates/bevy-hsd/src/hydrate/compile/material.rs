use bevy::prelude::*;
use bevy_wds::{BlobDeps, BlobDepsLoaded, BlobResponse};
use smol_str::SmolStr;

use crate::{CompiledMaterial, HsdChild, cache::SceneRegistry};

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
    pub base_color_texture: Option<Entity>,
    _metallic_roughness_texture: Option<Entity>,
    _normal_texture: Option<Entity>,
    _occlusion_texture: Option<Entity>,
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
    let ent = commands
        .spawn((HsdChild { doc: ev.doc }, MaterialParams::default()))
        .id();
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
    let ent = *inner.entity.lock().expect("entity lock");
    if let Some(ent) = ent {
        commands.entity(ent).despawn();
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
    if let Some(ref name) = ev.name {
        commands.entity(ent).insert(Name::new(name.clone()));
    } else {
        commands.entity(ent).remove::<Name>();
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

pub(crate) fn on_material_blobs_loaded(
    trigger: On<Add, BlobDepsLoaded>,
    mat_params: Query<(&MaterialParams, Option<&CompiledMaterial>)>,
    mut mat_assets: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut blobs: Query<&mut BlobResponse>,
) {
    let ent = trigger.entity;
    let Ok((params, existing)) = mat_params.get(ent) else {
        return;
    };

    let mut material = StandardMaterial::default();

    if let Some(value) = params.base_color {
        material.base_color = value;
    }
    let alpha_mode = match params.alpha_mode.as_deref() {
        Some("blend") => AlphaMode::Blend,
        Some("mask") => AlphaMode::Mask(params.alpha_cutoff.unwrap_or(0.5)),
        Some("opaque") => AlphaMode::Opaque,
        _ => {
            let alpha = material.base_color.alpha();
            if alpha < 1.0 {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            }
        }
    };
    material.alpha_mode = alpha_mode;
    if let Some(value) = params.double_sided {
        material.double_sided = value;
    }
    if let Some(value) = params.metallic {
        material.metallic = value;
    }
    if let Some(value) = params.roughness {
        material.perceptual_roughness = value;
    }

    if let Some(value) = params.base_color_texture {
        let Ok(Some(_bytes)) = blobs.get_mut(value).map(|mut b| b.0.take()) else {
            return;
        };
        // TODO: load image details from HSD
    }

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
        let handle = asset_server.add(material);
        commands
            .entity(ent)
            .insert(CompiledMaterial(handle))
            .remove::<BlobDeps>()
            .remove::<BlobDepsLoaded>();
    }
}
