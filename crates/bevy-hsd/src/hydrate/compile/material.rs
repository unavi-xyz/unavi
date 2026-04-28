use bevy::prelude::*;
use bevy_wds::blob::deps::{BlobDeps, BlobDepsLoaded};
use smol_str::SmolStr;

use hsd::HsdMaterial;

use crate::{
    DocRegistryMap, HsdChild, HsdEntityMaps, MaterialId, hydrate::compile::image::CompiledImage,
};

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
    mut entity_maps: Query<&mut HsdEntityMaps>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "material spawned");
    let Some(doc_ent) = registry_map.get_entity(&ev.doc_id) else {
        return;
    };
    let Ok(mut maps) = entity_maps.get_mut(doc_ent) else {
        return;
    };
    if maps.materials.contains_key(&ev.id) {
        return;
    }

    let params = ev
        .initial
        .as_ref()
        .map(material_params_from_hsd)
        .unwrap_or_default();

    let ent = commands
        .spawn((HsdChild { doc: doc_ent }, MaterialId(ev.id.clone())))
        .id();
    commands.entity(ent).insert(params);
    maps.materials.insert(ev.id.clone(), ent);
}

pub(crate) fn handle_hsd_material_despawned(
    trigger: On<HsdMaterialDespawned>,
    registry_map: Res<DocRegistryMap>,
    mut entity_maps: Query<&mut HsdEntityMaps>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "material despawned");
    let Some(doc_ent) = registry_map.get_entity(&ev.doc_id) else {
        return;
    };
    let Ok(mut maps) = entity_maps.get_mut(doc_ent) else {
        return;
    };
    let Some(ent) = maps.materials.remove(&ev.id) else {
        return;
    };
    if let Ok(mut entity_cmd) = commands.get_entity(ent) {
        entity_cmd.despawn();
    }
}

fn get_material_entity(
    registry_map: &DocRegistryMap,
    entity_maps: &Query<&HsdEntityMaps>,
    doc_id: &blake3::Hash,
    id: &SmolStr,
) -> Option<Entity> {
    let doc_ent = registry_map.get_entity(doc_id)?;
    let maps = entity_maps.get(doc_ent).ok()?;
    maps.materials.get(id).copied()
}

fn get_image_entity(
    registry_map: &DocRegistryMap,
    entity_maps: &Query<&HsdEntityMaps>,
    doc_id: &blake3::Hash,
    id: &SmolStr,
) -> Option<Entity> {
    let doc_ent = registry_map.get_entity(doc_id)?;
    let maps = entity_maps.get(doc_ent).ok()?;
    maps.images.get(id).copied()
}

fn update_material_param(
    registry_map: &DocRegistryMap,
    entity_maps: &Query<&HsdEntityMaps>,
    doc_id: &blake3::Hash,
    id: &SmolStr,
    params: &mut Query<&mut MaterialParams>,
    f: impl FnOnce(&mut MaterialParams),
) {
    let Some(ent) = get_material_entity(registry_map, entity_maps, doc_id, id) else {
        return;
    };
    if let Ok(mut p) = params.get_mut(ent) {
        f(&mut p);
    }
}

fn update_material_texture(
    registry_map: &DocRegistryMap,
    entity_maps: &Query<&HsdEntityMaps>,
    doc_id: &blake3::Hash,
    mat_id: &SmolStr,
    image_id: &SmolStr,
    params: &mut Query<&mut MaterialParams>,
    f: impl FnOnce(&mut MaterialParams, Option<Entity>),
) {
    let Some(ent) = get_material_entity(registry_map, entity_maps, doc_id, mat_id) else {
        return;
    };
    let img_ent = get_image_entity(registry_map, entity_maps, doc_id, image_id);
    if let Ok(mut p) = params.get_mut(ent) {
        f(&mut p, img_ent);
    }
}

pub(crate) fn handle_hsd_material_alpha_cutoff_set(
    trigger: On<HsdMaterialAlphaCutoffSet>,
    registry_map: Res<DocRegistryMap>,
    entity_maps: Query<&HsdEntityMaps>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    update_material_param(
        &registry_map,
        &entity_maps,
        &ev.doc_id,
        &ev.id,
        &mut params,
        |p| {
            p.alpha_cutoff = Some(ev.value);
        },
    );
}

pub(crate) fn handle_hsd_material_alpha_mode_set(
    trigger: On<HsdMaterialAlphaModeSet>,
    registry_map: Res<DocRegistryMap>,
    entity_maps: Query<&HsdEntityMaps>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    update_material_param(
        &registry_map,
        &entity_maps,
        &ev.doc_id,
        &ev.id,
        &mut params,
        |p| {
            p.alpha_mode.clone_from(&ev.mode);
        },
    );
}

pub(crate) fn handle_hsd_material_base_color_set(
    trigger: On<HsdMaterialBaseColorSet>,
    registry_map: Res<DocRegistryMap>,
    entity_maps: Query<&HsdEntityMaps>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    update_material_param(
        &registry_map,
        &entity_maps,
        &ev.doc_id,
        &ev.id,
        &mut params,
        |p| {
            let [r, g, b, a] = ev.color;
            p.base_color = Some(Color::srgba(r, g, b, a));
        },
    );
}

pub(crate) fn handle_hsd_material_base_color_texture_set(
    trigger: On<HsdMaterialBaseColorTextureSet>,
    registry_map: Res<DocRegistryMap>,
    entity_maps: Query<&HsdEntityMaps>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    update_material_texture(
        &registry_map,
        &entity_maps,
        &ev.doc_id,
        &ev.id,
        &ev.value,
        &mut params,
        |p, img| p.base_color_texture = img,
    );
}

pub(crate) fn handle_hsd_material_emissive_texture_set(
    trigger: On<HsdMaterialEmissiveTextureSet>,
    registry_map: Res<DocRegistryMap>,
    entity_maps: Query<&HsdEntityMaps>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    update_material_texture(
        &registry_map,
        &entity_maps,
        &ev.doc_id,
        &ev.id,
        &ev.value,
        &mut params,
        |p, img| p.emissive_texture = img,
    );
}

pub(crate) fn handle_hsd_material_metallic_roughness_texture_set(
    trigger: On<HsdMaterialMetallicRoughnessTextureSet>,
    registry_map: Res<DocRegistryMap>,
    entity_maps: Query<&HsdEntityMaps>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    update_material_texture(
        &registry_map,
        &entity_maps,
        &ev.doc_id,
        &ev.id,
        &ev.value,
        &mut params,
        |p, img| p.metallic_roughness_texture = img,
    );
}

pub(crate) fn handle_hsd_material_normal_texture_set(
    trigger: On<HsdMaterialNormalTextureSet>,
    registry_map: Res<DocRegistryMap>,
    entity_maps: Query<&HsdEntityMaps>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    update_material_texture(
        &registry_map,
        &entity_maps,
        &ev.doc_id,
        &ev.id,
        &ev.value,
        &mut params,
        |p, img| p.normal_texture = img,
    );
}

pub(crate) fn handle_hsd_material_occlusion_texture_set(
    trigger: On<HsdMaterialOcclusionTextureSet>,
    registry_map: Res<DocRegistryMap>,
    entity_maps: Query<&HsdEntityMaps>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    update_material_texture(
        &registry_map,
        &entity_maps,
        &ev.doc_id,
        &ev.id,
        &ev.value,
        &mut params,
        |p, img| p.occlusion_texture = img,
    );
}

pub(crate) fn handle_hsd_material_double_sided_set(
    trigger: On<HsdMaterialDoubleSidedSet>,
    registry_map: Res<DocRegistryMap>,
    entity_maps: Query<&HsdEntityMaps>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    update_material_param(
        &registry_map,
        &entity_maps,
        &ev.doc_id,
        &ev.id,
        &mut params,
        |p| {
            p.double_sided = Some(ev.value);
        },
    );
}

pub(crate) fn handle_hsd_material_metallic_set(
    trigger: On<HsdMaterialMetallicSet>,
    registry_map: Res<DocRegistryMap>,
    entity_maps: Query<&HsdEntityMaps>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    update_material_param(
        &registry_map,
        &entity_maps,
        &ev.doc_id,
        &ev.id,
        &mut params,
        |p| {
            p.metallic = Some(ev.value);
        },
    );
}

pub(crate) fn handle_hsd_material_name_set(
    trigger: On<HsdMaterialNameSet>,
    registry_map: Res<DocRegistryMap>,
    entity_maps: Query<&HsdEntityMaps>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    let Some(ent) = get_material_entity(&registry_map, &entity_maps, &ev.doc_id, &ev.id) else {
        return;
    };
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
    entity_maps: Query<&HsdEntityMaps>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    update_material_param(
        &registry_map,
        &entity_maps,
        &ev.doc_id,
        &ev.id,
        &mut params,
        |p| {
            p.unlit = Some(ev.value);
        },
    );
}

pub(crate) fn handle_hsd_material_roughness_set(
    trigger: On<HsdMaterialRoughnessSet>,
    registry_map: Res<DocRegistryMap>,
    entity_maps: Query<&HsdEntityMaps>,
    mut params: Query<&mut MaterialParams>,
) {
    let ev = trigger.event();
    update_material_param(
        &registry_map,
        &entity_maps,
        &ev.doc_id,
        &ev.id,
        &mut params,
        |p| {
            p.roughness = Some(ev.value);
        },
    );
}

const METALLIC_DEFAULT: f32 = 0.5;
const ROUGHNESS_DEFAULT: f32 = 0.5;

fn build_standard_material(
    material: &mut StandardMaterial,
    params: &MaterialParams,
    compiled_images: &Query<&CompiledImage>,
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
    material.metallic = params.metallic.unwrap_or(METALLIC_DEFAULT);
    material.perceptual_roughness = params.roughness.unwrap_or(ROUGHNESS_DEFAULT);
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
    compiled_images: Query<&CompiledImage>,
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
    compiled_images: Query<&CompiledImage>,
    mut mat_assets: ResMut<Assets<StandardMaterial>>,
) {
    for (params, compiled) in &changed {
        let Some(material) = mat_assets.get_mut(&compiled.0) else {
            continue;
        };

        build_standard_material(material, params, &compiled_images);
    }
}
