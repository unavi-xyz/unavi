use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_wds::{BlobDep, BlobDeps, BlobDepsLoaded, BlobRequest, BlobResponse};
use blake3::Hash;
use image::GenericImageView;
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
pub struct HsdMaterialBaseColorTextureSet {
    pub doc: Entity,
    pub id: SmolStr,
    pub value: Hash,
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

#[derive(Component, Default, Debug)]
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
    pub metallic_roughness_texture: Option<Entity>,
    pub normal_texture: Option<Entity>,
    pub occlusion_texture: Option<Entity>,
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

    let mut params = ev
        .initial
        .as_ref()
        .map(material_params_from_hsd)
        .unwrap_or_default();

    let entity = commands.spawn(HsdChild { doc: ev.doc }).id();

    if let Some(val) = &ev.initial {
        if let Some(h) = val.base_color_texture.map(|h| h.0) {
            let blob_ent = commands
                .spawn((BlobRequest(h), BlobDep { owner: entity }))
                .id();
            params.base_color_texture = Some(blob_ent);
        }
        if let Some(h) = val.normal_texture.map(|h| h.0) {
            let blob_ent = commands
                .spawn((BlobRequest(h), BlobDep { owner: entity }))
                .id();
            params.normal_texture = Some(blob_ent);
        }
        if let Some(h) = val.metallic_roughness_texture.map(|h| h.0) {
            let blob_ent = commands
                .spawn((BlobRequest(h), BlobDep { owner: entity }))
                .id();
            params.metallic_roughness_texture = Some(blob_ent);
        }
        if let Some(h) = val.occlusion_texture.map(|h| h.0) {
            let blob_ent = commands
                .spawn((BlobRequest(h), BlobDep { owner: entity }))
                .id();
            params.occlusion_texture = Some(blob_ent);
        }
    }

    commands.entity(entity).insert(params);
    *inner.entity.lock().expect("entity lock") = Some(entity);
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

pub(crate) fn handle_hsd_material_base_color_texture_set(
    trigger: On<HsdMaterialBaseColorTextureSet>,
    registries: Query<&SceneRegistry>,
    mut params: Query<&mut MaterialParams>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, hash = %ev.value, "material base color texture set");
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
        let blob_ent = commands
            .spawn((BlobRequest(ev.value), BlobDep { owner: ent }))
            .id();
        p.base_color_texture = Some(blob_ent);
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

const MAX_TEXTURE_DIMS: u32 = 8192;

fn build_standard_material(
    material: &mut StandardMaterial,
    params: &MaterialParams,
    blobs: &mut Query<&mut BlobResponse>,
    images: &mut ResMut<Assets<Image>>,
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

    if let Some(value) = params.base_color_texture
        && let Ok(Some(bytes)) = blobs.get_mut(value).map(|mut b| b.0.take())
        && let Ok(dyn_img) = image::load_from_memory(&bytes)
    {
        let (width, height) = dyn_img.dimensions();
        let rgba = dyn_img.into_rgba8();

        if width > MAX_TEXTURE_DIMS || height > MAX_TEXTURE_DIMS {
            warn!("texture too large: {width}x{height}");
            return;
        }

        let img = Image::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            rgba.into_raw(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );

        // TODO load sampler from hsd

        let handle = images.add(img);
        material.base_color_texture = Some(handle);
    } else {
        material.base_color_texture = None;
    }

    // TODO load other textures
}

pub(crate) fn on_material_blobs_loaded(
    trigger: On<Add, BlobDepsLoaded>,
    mat_params: Query<(&MaterialParams, Option<&CompiledMaterial>)>,
    mut mat_assets: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut blobs: Query<&mut BlobResponse>,
    mut images: ResMut<Assets<Image>>,
) {
    let entity = trigger.entity;
    let Ok((params, existing)) = mat_params.get(entity) else {
        return;
    };

    let mut material = StandardMaterial::default();
    build_standard_material(&mut material, params, &mut blobs, &mut images);

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
    mut blobs: Query<&mut BlobResponse>,
    mut images: ResMut<Assets<Image>>,
    mut mat_assets: ResMut<Assets<StandardMaterial>>,
) {
    for (params, compiled) in &changed {
        let Some(material) = mat_assets.get_mut(&compiled.0) else {
            continue;
        };

        build_standard_material(material, params, &mut blobs, &mut images);
    }
}
