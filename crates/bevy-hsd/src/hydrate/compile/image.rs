//! Compiles HSD image data into Bevy `Image` assets.
//!
//! Raw bytes are fetched asynchronously via the blob-dep system. Once compiled,
//! `on_image_compiled` pushes the handle into any already-compiled materials
//! that reference this image.

use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_wds::blob::{
    deps::{BlobDep, BlobDeps, BlobDepsLoaded},
    request::{BlobRequest, BlobResponse},
};
use image::{DynamicImage, GenericImageView};
use smol_str::SmolStr;

use crate::{
    DocRegistryMap, HsdChild,
    data::HsdImage,
    hydrate::compile::material::{CompiledMaterial, MaterialParams},
};

/// Marks an image entity as having a ready `Handle<Image>`.
#[derive(Component)]
pub struct CompiledImage(pub Handle<Image>);

/// Stable HSD image key kept on the entity for lookup by materials.
#[derive(Component, Clone, Debug)]
pub struct ImageId(pub SmolStr);

#[derive(Event)]
pub struct HsdImageSpawned {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
    pub initial: Option<HsdImage>,
}

#[derive(Event)]
pub struct HsdImageDespawned {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
}

/// Sampler config and a reference to the blob entity carrying the raw bytes.
/// `srgb: None` defaults to sRGB; set `Some(false)` for linear textures.
#[derive(Component, Default, Debug)]
#[require(BlobDeps)]
pub struct ImageParams {
    pub address_mode_u: Option<ImageAddressMode>,
    pub address_mode_v: Option<ImageAddressMode>,
    pub address_mode_w: Option<ImageAddressMode>,
    pub data: Option<Entity>,
    pub mag_filter: Option<ImageFilterMode>,
    pub min_filter: Option<ImageFilterMode>,
    pub mipmap_filter: Option<ImageFilterMode>,
    pub name: Option<SmolStr>,
    pub srgb: Option<bool>,
}

const MAX_TEXTURE_DIMS: u32 = 8192;

const fn address_mode(v: i64) -> ImageAddressMode {
    match v {
        1 => ImageAddressMode::MirrorRepeat,
        2 => ImageAddressMode::ClampToEdge,
        _ => ImageAddressMode::Repeat,
    }
}

const fn filter_mode(v: i64) -> ImageFilterMode {
    match v {
        1 => ImageFilterMode::Nearest,
        _ => ImageFilterMode::Linear,
    }
}

fn params_from_hsd(hsd: &HsdImage) -> ImageParams {
    ImageParams {
        address_mode_u: hsd.address_mode_u.map(address_mode),
        address_mode_v: hsd.address_mode_v.map(address_mode),
        address_mode_w: hsd.address_mode_w.map(address_mode),
        data: None,
        mag_filter: hsd.mag_filter.map(filter_mode),
        min_filter: hsd.min_filter.map(filter_mode),
        mipmap_filter: hsd.mipmap_filter.map(filter_mode),
        name: hsd.name.clone(),
        srgb: hsd.srgb,
    }
}

pub(crate) fn build_img(dyn_img: DynamicImage, params: &ImageParams) -> Image {
    let (width, height) = dyn_img.dimensions();
    let rgba = dyn_img.into_rgba8();

    if width > MAX_TEXTURE_DIMS || height > MAX_TEXTURE_DIMS {
        warn!("image too large: {width}x{height}");
        return Image::default();
    }

    let format = if params.srgb == Some(false) {
        TextureFormat::Rgba8Unorm
    } else {
        TextureFormat::Rgba8UnormSrgb
    };

    let mut img = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba.into_raw(),
        format,
        RenderAssetUsages::default(),
    );

    let mut sampler = ImageSamplerDescriptor::default();

    if let Some(value) = params.address_mode_u {
        sampler.address_mode_u = value;
    }
    if let Some(value) = params.address_mode_v {
        sampler.address_mode_v = value;
    }
    if let Some(value) = params.address_mode_w {
        sampler.address_mode_w = value;
    }
    if let Some(value) = params.mag_filter {
        sampler.mag_filter = value;
    }
    if let Some(value) = params.min_filter {
        sampler.min_filter = value;
    }
    if let Some(value) = params.mipmap_filter {
        sampler.mipmap_filter = value;
    }

    img.sampler = ImageSampler::Descriptor(sampler);

    img
}

pub(crate) fn handle_hsd_image_spawned(
    trigger: On<HsdImageSpawned>,
    registry_map: Res<DocRegistryMap>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "image spawned");
    let Some((doc_entity, registry)) = registry_map.0.get(&ev.doc_id) else {
        return;
    };
    let inner = registry
        .images
        .lock()
        .expect("images lock")
        .get(&ev.id)
        .cloned();
    let Some(inner) = inner else { return };

    let mut params = ev.initial.as_ref().map(params_from_hsd).unwrap_or_default();

    let existing = *inner.entity.lock().expect("entity lock");
    let entity = existing.unwrap_or_else(|| {
        let ent = commands
            .spawn((HsdChild { doc: *doc_entity }, ImageId(ev.id.clone())))
            .id();
        *inner.entity.lock().expect("entity lock") = Some(ent);
        ent
    });

    if let Some(ref hsd) = ev.initial
        && let Some(ref hash) = hsd.data
    {
        let blob_ent = commands.spawn((BlobRequest(hash.0), BlobDep(entity))).id();
        params.data = Some(blob_ent);
    }

    commands.entity(entity).insert(params);
}

pub(crate) fn handle_hsd_image_despawned(
    trigger: On<HsdImageDespawned>,
    registry_map: Res<DocRegistryMap>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "image despawned");
    let Some((_, registry)) = registry_map.0.get(&ev.doc_id) else {
        return;
    };
    let inner = registry.images.lock().expect("images lock").remove(&ev.id);
    let Some(inner) = inner else { return };
    if let Some(ent) = *inner.entity.lock().expect("entity lock")
        && let Ok(mut ent) = commands.get_entity(ent)
    {
        ent.despawn();
    }
}

pub(crate) fn on_image_blobs_loaded(
    trigger: On<Add, BlobDepsLoaded>,
    img_params: Query<(&ImageParams, Option<&CompiledImage>)>,
    mut img_assets: ResMut<Assets<Image>>,
    mut commands: Commands,
    mut blobs: Query<&mut BlobResponse>,
) {
    let entity = trigger.entity;
    let Ok((params, existing)) = img_params.get(entity) else {
        return;
    };

    let img = if let Some(data_ent) = params.data
        && let Ok(Some(bytes)) = blobs.get_mut(data_ent).map(|mut b| b.0.take())
        && let Ok(dyn_img) = image::load_from_memory(&bytes)
    {
        build_img(dyn_img, params)
    } else {
        Image::default()
    };

    debug!("compiled image {entity}");
    if let Some(CompiledImage(handle)) = existing {
        if let Some(asset) = img_assets.get_mut(handle) {
            *asset = img;
            commands
                .entity(entity)
                .remove::<BlobDeps>()
                .remove::<BlobDepsLoaded>();
        }
    } else {
        let handle = img_assets.add(img);
        commands
            .entity(entity)
            .insert(CompiledImage(handle))
            .remove::<BlobDeps>()
            .remove::<BlobDepsLoaded>();
    }
}

/// When an image entity compiles, update any materials referencing it.
pub(crate) fn on_image_compiled(
    trigger: On<Add, CompiledImage>,
    compiled_images: Query<&CompiledImage>,
    mat_params: Query<(Entity, &MaterialParams)>,
    compiled_mats: Query<&CompiledMaterial>,
    mut mat_assets: ResMut<Assets<StandardMaterial>>,
) {
    let img_ent = trigger.entity;
    let Ok(compiled) = compiled_images.get(img_ent) else {
        return;
    };

    for (mat_ent, params) in &mat_params {
        let references_this = [
            params.base_color_texture,
            params.emissive_texture,
            params.metallic_roughness_texture,
            params.normal_texture,
            params.occlusion_texture,
        ]
        .contains(&Some(img_ent));

        if !references_this {
            continue;
        }

        let Ok(compiled_mat) = compiled_mats.get(mat_ent) else {
            continue;
        };
        let Some(material) = mat_assets.get_mut(&compiled_mat.0) else {
            continue;
        };

        let handle = compiled.0.clone();

        if params.base_color_texture == Some(img_ent) {
            material.base_color_texture = Some(handle.clone());
        }
        if params.emissive_texture == Some(img_ent) {
            material.emissive_texture = Some(handle.clone());
        }
        if params.metallic_roughness_texture == Some(img_ent) {
            material.metallic_roughness_texture = Some(handle.clone());
        }
        if params.normal_texture == Some(img_ent) {
            material.normal_map_texture = Some(handle.clone());
        }
        if params.occlusion_texture == Some(img_ent) {
            material.occlusion_texture = Some(handle.clone());
        }
    }
}
