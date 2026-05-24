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
use hsd::{
    HSD_CONTAINER_ID,
    attributes::{Attribute, hydrate_attr, image::ImageAttr},
};
use image::GenericImageView;
use loro::{ContainerID, Index, TreeID, ValueOrContainer, event::Diff};

use crate::{
    attributes::{
        ApplyEvent, AttrDataEvent, AttributeParser, DocContext, ParseError,
        util::shallow_map_updated_keys,
    },
    diff::HsdDiffEvent,
};

const MAX_TEXTURE_DIMS: u32 = 8192;

#[derive(Debug)]
pub enum ImageEvent {
    Rebuild(ImageAttr),
}

#[derive(Component)]
#[require(BlobDeps)]
pub struct ImageBlobs {
    pub data: Entity,
    pub sampler: ImageSamplerDescriptor,
    pub srgb: Option<bool>,
}

#[derive(Component)]
#[relationship(relationship_target = ImageBlobsChild)]
pub struct ImageBlobsOwner(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = ImageBlobsOwner, linked_spawn)]
pub struct ImageBlobsChild(Entity);

#[derive(Component, Default)]
pub struct HsdImage(pub Handle<Image>);

pub struct ImageParser;

impl AttributeParser for ImageParser {
    fn key(&self) -> &'static str {
        ImageAttr::KEY
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        value: Option<ValueOrContainer>,
    ) -> Result<(), ParseError> {
        if value.is_some() {
            commands.entity(prim).insert(HsdImage::default());
        } else {
            commands
                .entity(prim)
                .remove::<(HsdImage, ImageBlobsChild)>();
        }
        Ok(())
    }

    fn parse(
        &self,
        ctx: &DocContext,
        prim: TreeID,
        path: &[(ContainerID, Index)],
        diff: Diff,
    ) -> Result<(), ParseError> {
        let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
        let meta = tree.get_meta(prim)?;

        let attr: ImageAttr = hydrate_attr(&meta)?;

        let keys = shallow_map_updated_keys(path, diff)?;
        if keys.is_empty() {
            return Ok(());
        }

        ctx.tx
            .send(HsdDiffEvent::AttrData {
                prim,
                data: AttrDataEvent::Image(ImageEvent::Rebuild(attr)),
            })
            .map_err(|_| ParseError::SendDiff)?;
        Ok(())
    }
}

pub fn apply_image(trigger: On<ApplyEvent<ImageEvent>>, mut commands: Commands) {
    let prim = trigger.entity;
    let ImageEvent::Rebuild(attr) = &trigger.value;

    commands.entity(prim).remove::<ImageBlobsChild>();

    let child = commands.spawn(ImageBlobsOwner(prim)).id();

    let data = commands
        .spawn((
            BlobDep(child),
            BlobRequest(blake3::Hash::from_bytes(attr.data.0)),
        ))
        .id();

    let mut sampler = ImageSamplerDescriptor::default();
    if let Some(&v) = attr.address_mode_u.as_ref() {
        sampler.address_mode_u = address_mode(v);
    }
    if let Some(&v) = attr.address_mode_v.as_ref() {
        sampler.address_mode_v = address_mode(v);
    }
    if let Some(&v) = attr.address_mode_w.as_ref() {
        sampler.address_mode_w = address_mode(v);
    }
    if let Some(&v) = attr.mag_filter.as_ref() {
        sampler.mag_filter = filter_mode(v);
    }
    if let Some(&v) = attr.min_filter.as_ref() {
        sampler.min_filter = filter_mode(v);
    }
    if let Some(&v) = attr.mipmap_filter.as_ref() {
        sampler.mipmap_filter = filter_mode(v);
    }

    commands.entity(child).insert(ImageBlobs {
        data,
        sampler,
        srgb: attr.srgb.as_ref().copied(),
    });
}

pub fn on_image_blob_loaded(
    trigger: On<Add, BlobDepsLoaded>,
    image_blobs: Query<(&ImageBlobs, &ImageBlobsOwner)>,
    mut blob_responses: Query<&mut BlobResponse>,
    mut hsd_images: Query<&mut HsdImage>,
    mut image_assets: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    let child = trigger.entity;
    let Ok((params, owner)) = image_blobs.get(child) else {
        return;
    };
    let prim = owner.0;

    let Ok(Some(bytes)) = blob_responses.get_mut(params.data).map(|mut b| b.0.take()) else {
        warn!("image blob not found");
        commands.entity(child).try_despawn();
        return;
    };

    let dyn_img = match image::load_from_memory(&bytes) {
        Ok(img) => img,
        Err(err) => {
            warn!(?err, "failed to decode image");
            commands.entity(child).try_despawn();
            return;
        }
    };

    let img = build_img(dyn_img, params.sampler.clone(), params.srgb);
    let handle = image_assets.add(img);

    if let Ok(mut hsd_image) = hsd_images.get_mut(prim) {
        hsd_image.0 = handle;
    } else {
        commands.entity(prim).insert(HsdImage(handle));
    }

    commands.entity(child).try_despawn();
}

fn build_img(
    dyn_img: image::DynamicImage,
    sampler: ImageSamplerDescriptor,
    srgb: Option<bool>,
) -> Image {
    let (width, height) = dyn_img.dimensions();
    if width > MAX_TEXTURE_DIMS || height > MAX_TEXTURE_DIMS {
        warn!("image too large: {width}x{height}");
        return Image::default();
    }

    let rgba = dyn_img.into_rgba8();
    let format = if srgb == Some(false) {
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

    img.sampler = ImageSampler::Descriptor(sampler);

    img
}

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
