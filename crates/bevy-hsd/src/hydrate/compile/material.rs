//! Compiles HSD material data into Bevy `StandardMaterial` assets.
//!
//! Texture slots store image entity refs rather than blob hashes; the image
//! entity supplies the `Handle<Image>` once it has compiled. If an image
//! compiles after the material, `on_image_compiled` (in `compile::image`)
//! patches the already-live `StandardMaterial` directly.

use bevy::prelude::*;
use bevy_wds::{BlobDeps, BlobDepsLoaded};
use smol_str::SmolStr;

use crate::{DocRegistryMap, HsdChild, cache::SceneRegistryInner, data::HsdMaterial};

/// Marks a material entity as having a ready `Handle<StandardMaterial>`.
#[derive(Component)]
pub struct CompiledMaterial(pub Handle<StandardMaterial>);

#[derive(Event)]
pub struct HsdMaterialAlphaCutoffSet {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
    pub value: f32,
}

#[derive(Event)]
pub struct HsdMaterialAlphaModeSet {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
    pub mode: Option<String>,
}

#[derive(Event)]
pub struct HsdMaterialBaseColorSet {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
    pub color: [f32; 4],
}

#[derive(Event)]
pub struct HsdMaterialBaseColorTextureSet {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
    pub value: SmolStr,
}

#[derive(Event)]
pub struct HsdMaterialEmissiveTextureSet {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
    pub value: SmolStr,
}

#[derive(Event)]
pub struct HsdMaterialMetallicRoughnessTextureSet {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
    pub value: SmolStr,
}

#[derive(Event)]
pub struct HsdMaterialNormalTextureSet {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
    pub value: SmolStr,
}

#[derive(Event)]
pub struct HsdMaterialOcclusionTextureSet {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
    pub value: SmolStr,
}

#[derive(Event)]
pub struct HsdMaterialDespawned {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
}

#[derive(Event)]
pub struct HsdMaterialDoubleSidedSet {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
    pub value: bool,
}

#[derive(Event)]
pub struct HsdMaterialMetallicSet {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
    pub value: f32,
}

#[derive(Event)]
pub struct HsdMaterialNameSet {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
    pub name: Option<String>,
}

#[derive(Event)]
pub struct HsdMaterialRoughnessSet {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
    pub value: f32,
}

#[derive(Event)]
pub struct HsdMaterialSpawned {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
    pub initial: Option<HsdMaterial>,
}

#[derive(Event)]
pub struct HsdMaterialUnlitSet {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
    pub value: bool,
}

/// All material properties. Texture fields hold image entity refs; the handle
/// is resolved at compile time via `CompiledImage` on the image entity.
#[derive(Component, Default, Debug)]
#[require(BlobDeps)]
pub struct MaterialParams {
    pub alpha_cutoff: Option<f32>,
    pub alpha_mode: Option<String>,
    pub base_color: Option<Color>,
    pub base_color_texture: Option<Entity>,
    pub double_sided: Option<bool>,
    pub emissive_texture: Option<Entity>,
    pub metallic: Option<f32>,
    pub metallic_roughness_texture: Option<Entity>,
    pub normal_texture: Option<Entity>,
    pub occlusion_texture: Option<Entity>,
    pub roughness: Option<f32>,
    pub unlit: Option<bool>,
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
    registry_map: Res<DocRegistryMap>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "material spawned");
    let Some((doc_entity, registry)) = registry_map.0.get(&ev.doc_id) else {
        return;
    };
    let inner = registry
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

    let entity = commands.spawn(HsdChild { doc: *doc_entity }).id();

    commands.entity(entity).insert(params);
    *inner.entity.lock().expect("entity lock") = Some(entity);
}

pub(crate) fn handle_hsd_material_despawned(
    trigger: On<HsdMaterialDespawned>,
    registry_map: Res<DocRegistryMap>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "material despawned");
    let Some((_, registry)) = registry_map.0.get(&ev.doc_id) else {
        return;
    };
    let inner = {
        let mut mats = registry.materials.lock().expect("materials lock");
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
    registry_map: Res<DocRegistryMap>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, value = ev.value, "material alpha cutoff set");
    let Some((_, registry)) = registry_map.0.get(&ev.doc_id) else {
        return;
    };
    let ent = registry
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
    registry_map: Res<DocRegistryMap>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, mode = ?ev.mode, "material alpha mode set");
    let Some((_, registry)) = registry_map.0.get(&ev.doc_id) else {
        return;
    };
    let ent = registry
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
    registry_map: Res<DocRegistryMap>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, color = ?ev.color, "material base color set");
    let Some((_, registry)) = registry_map.0.get(&ev.doc_id) else {
        return;
    };
    let ent = registry
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

fn get_image_entity(registry: &SceneRegistryInner, image_id: &SmolStr) -> Option<Entity> {
    registry
        .images
        .lock()
        .expect("images lock")
        .get(image_id)
        .and_then(|i| *i.entity.lock().expect("entity lock"))
}

fn get_mat_entity(registry: &SceneRegistryInner, mat_id: &SmolStr) -> Option<Entity> {
    registry
        .materials
        .lock()
        .expect("materials lock")
        .get(mat_id)
        .and_then(|m| *m.entity.lock().expect("entity lock"))
}

pub(crate) fn handle_hsd_material_base_color_texture_set(
    trigger: On<HsdMaterialBaseColorTextureSet>,
    registry_map: Res<DocRegistryMap>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, image = %ev.value, "material base color texture set");
    let Some((_, registry)) = registry_map.0.get(&ev.doc_id) else {
        return;
    };
    let Some(mat_ent) = get_mat_entity(registry, &ev.id) else {
        return;
    };
    if let Ok(mut p) = params.get_mut(mat_ent) {
        p.base_color_texture = get_image_entity(registry, &ev.value);
    }
}

pub(crate) fn handle_hsd_material_emissive_texture_set(
    trigger: On<HsdMaterialEmissiveTextureSet>,
    registry_map: Res<DocRegistryMap>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, image = %ev.value, "material emissive texture set");
    let Some((_, registry)) = registry_map.0.get(&ev.doc_id) else {
        return;
    };
    let Some(mat_ent) = get_mat_entity(registry, &ev.id) else {
        return;
    };
    if let Ok(mut p) = params.get_mut(mat_ent) {
        p.emissive_texture = get_image_entity(registry, &ev.value);
    }
}

pub(crate) fn handle_hsd_material_metallic_roughness_texture_set(
    trigger: On<HsdMaterialMetallicRoughnessTextureSet>,
    registry_map: Res<DocRegistryMap>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, image = %ev.value, "material metallic roughness texture set");
    let Some((_, registry)) = registry_map.0.get(&ev.doc_id) else {
        return;
    };
    let Some(mat_ent) = get_mat_entity(registry, &ev.id) else {
        return;
    };
    if let Ok(mut p) = params.get_mut(mat_ent) {
        p.metallic_roughness_texture = get_image_entity(registry, &ev.value);
    }
}

pub(crate) fn handle_hsd_material_normal_texture_set(
    trigger: On<HsdMaterialNormalTextureSet>,
    registry_map: Res<DocRegistryMap>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, image = %ev.value, "material normal texture set");
    let Some((_, registry)) = registry_map.0.get(&ev.doc_id) else {
        return;
    };
    let Some(mat_ent) = get_mat_entity(registry, &ev.id) else {
        return;
    };
    if let Ok(mut p) = params.get_mut(mat_ent) {
        p.normal_texture = get_image_entity(registry, &ev.value);
    }
}

pub(crate) fn handle_hsd_material_occlusion_texture_set(
    trigger: On<HsdMaterialOcclusionTextureSet>,
    registry_map: Res<DocRegistryMap>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, image = %ev.value, "material occlusion texture set");
    let Some((_, registry)) = registry_map.0.get(&ev.doc_id) else {
        return;
    };
    let Some(mat_ent) = get_mat_entity(registry, &ev.id) else {
        return;
    };
    if let Ok(mut p) = params.get_mut(mat_ent) {
        p.occlusion_texture = get_image_entity(registry, &ev.value);
    }
}

pub(crate) fn handle_hsd_material_double_sided_set(
    trigger: On<HsdMaterialDoubleSidedSet>,
    registry_map: Res<DocRegistryMap>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, value = ev.value, "material double sided set");
    let Some((_, registry)) = registry_map.0.get(&ev.doc_id) else {
        return;
    };
    let ent = registry
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
    registry_map: Res<DocRegistryMap>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, value = ev.value, "material metallic set");
    let Some((_, registry)) = registry_map.0.get(&ev.doc_id) else {
        return;
    };
    let ent = registry
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
    registry_map: Res<DocRegistryMap>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, name = ?ev.name, "material name set");
    let Some((_, registry)) = registry_map.0.get(&ev.doc_id) else {
        return;
    };
    let ent = registry
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
        entity_cmd.try_remove::<Name>();
    }
}

pub(crate) fn handle_hsd_material_unlit_set(
    trigger: On<HsdMaterialUnlitSet>,
    registry_map: Res<DocRegistryMap>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, value = ev.value, "material unlit set");
    let Some((_, registry)) = registry_map.0.get(&ev.doc_id) else {
        return;
    };
    let ent = registry
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
    registry_map: Res<DocRegistryMap>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, value = ev.value, "material roughness set");
    let Some((_, registry)) = registry_map.0.get(&ev.doc_id) else {
        return;
    };
    let ent = registry
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

fn build_standard_material(
    material: &mut StandardMaterial,
    params: &MaterialParams,
    compiled_images: &Query<&super::image::CompiledImage>,
) {
    material.base_color = params.base_color.unwrap_or_default();

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

    material.double_sided = params.double_sided.unwrap_or_default();
    material.metallic = params.metallic.unwrap_or(0.5);
    material.perceptual_roughness = params.roughness.unwrap_or(0.5);
    material.unlit = params.unlit.unwrap_or_default();

    material.base_color_texture = params
        .base_color_texture
        .and_then(|e| compiled_images.get(e).ok())
        .map(|ci| ci.0.clone());
    material.emissive_texture = params
        .emissive_texture
        .and_then(|e| compiled_images.get(e).ok())
        .map(|ci| ci.0.clone());
    material.metallic_roughness_texture = params
        .metallic_roughness_texture
        .and_then(|e| compiled_images.get(e).ok())
        .map(|ci| ci.0.clone());
    material.normal_map_texture = params
        .normal_texture
        .and_then(|e| compiled_images.get(e).ok())
        .map(|ci| ci.0.clone());
    material.occlusion_texture = params
        .occlusion_texture
        .and_then(|e| compiled_images.get(e).ok())
        .map(|ci| ci.0.clone());
}

pub(crate) fn on_material_blobs_loaded(
    trigger: On<Add, BlobDepsLoaded>,
    mat_params: Query<(&MaterialParams, Option<&CompiledMaterial>)>,
    compiled_images: Query<&super::image::CompiledImage>,
    mut mat_assets: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let entity = trigger.entity;
    let Ok((params, existing)) = mat_params.get(entity) else {
        return;
    };

    let mut material = StandardMaterial::default();
    build_standard_material(&mut material, params, &compiled_images);

    debug!("compiled material {entity}");
    if let Some(CompiledMaterial(handle)) = existing {
        if let Some(asset) = mat_assets.get_mut(handle) {
            *asset = material;
            commands
                .entity(entity)
                .remove::<BlobDeps>()
                .remove::<BlobDepsLoaded>();
        }
    } else {
        let handle = mat_assets.add(material);
        commands
            .entity(entity)
            .insert(CompiledMaterial(handle))
            .remove::<BlobDeps>()
            .remove::<BlobDepsLoaded>();
    }
}

pub(crate) fn recompile_changed_materials(
    changed: Query<(&MaterialParams, &CompiledMaterial), Changed<MaterialParams>>,
    compiled_images: Query<&super::image::CompiledImage>,
    mut mat_assets: ResMut<Assets<StandardMaterial>>,
) {
    for (params, compiled) in &changed {
        let Some(material) = mat_assets.get_mut(&compiled.0) else {
            continue;
        };

        build_standard_material(material, params, &compiled_images);
    }
}
