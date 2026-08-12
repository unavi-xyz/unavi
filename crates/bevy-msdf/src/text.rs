//! Turns [`MsdfText`] into one mesh per atlas page.
//!
//! The parent entity carries the style and transform; each page it draws
//! becomes a child mesh so every page samples the image that holds it. Glyphs
//! the atlas still has to generate are requested first and drawn as fallbacks
//! until they land.

use std::{
    hash::{
        DefaultHasher,
        Hash,
        Hasher,
    },
    sync::Arc,
};

use bevy::{
    asset::AssetId,
    image::Image,
    light::NotShadowCaster,
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_asset::RenderAssets,
        render_resource::{
            Extent3d,
            Origin3d,
            TexelCopyBufferLayout,
            TexelCopyTextureInfo,
            TextureAspect,
        },
        renderer::RenderQueue,
        texture::GpuImage,
    },
};
use image::RgbaImage;
use msdf::{
    layout::{
        Align,
        LayoutOpts,
        layout,
    },
    runtime::DirtyRect,
};
use smol_str::SmolStr;

use crate::{
    font::{
        DefaultFont,
        MsdfFont,
        page_image,
        render_glyph,
    },
    material::{
        MsdfMaterial,
        MsdfSettings,
    },
    mesh::{
        Anchor,
        page_meshes,
    },
};

/// A string drawn in the world. Split from [`MsdfStyle`] so a style change
/// never re-tessellates the mesh.
#[derive(Component, Debug, Clone)]
#[require(Transform, Visibility, MsdfStyle)]
pub struct MsdfText {
    pub value:       SmolStr,
    /// Em height, in metres. 0.02 is body text read at arm's length.
    pub size:        f32,
    pub align:       Align,
    pub anchor:      Anchor,
    /// Wrap width in metres. `None` breaks only on newlines.
    pub wrap:        Option<f32>,
    pub line_height: f32,
    /// `None` draws with [`DefaultFont`].
    pub font:        Option<Arc<MsdfFont>>,
}

impl Default for MsdfText {
    fn default() -> Self {
        Self {
            value:       SmolStr::default(),
            size:        0.02,
            align:       Align::Left,
            anchor:      Anchor::Baseline,
            wrap:        None,
            line_height: 1.0,
            font:        None,
        }
    }
}

/// What the mesh for a text currently draws, and where its page children are.
/// Rebuilt in place whenever the layout inputs or the atlas change; a missing
/// runtime means the text was just added.
#[derive(Component, Debug)]
pub(crate) struct TextRuntime {
    font:       Arc<MsdfFont>,
    /// True while some of the text's glyphs are still pending.
    pending:    bool,
    /// The characters that were missing at the last layout.
    missing:    Vec<char>,
    /// Hash of everything a layout depends on; a match means the geometry is
    /// still current.
    layout_key: u64,
    /// One child mesh per page the text draws.
    pages:      Vec<Entity>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Outline {
    pub color: Color,
    /// Fraction of the baked distance range the outline reaches out to.
    /// Beyond roughly 0.4 the field runs out of gradient and the edge breaks
    /// up.
    pub width: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct MsdfStyle {
    pub color:    Color,
    /// Keeps text legible over backgrounds the text did not choose.
    pub outline:  Option<Outline>,
    pub emissive: f32,
}

impl Default for MsdfStyle {
    fn default() -> Self {
        Self {
            color:    Color::WHITE,
            outline:  None,
            emissive: 0.0,
        }
    }
}

/// Characters the font had no glyph for, per entity; [`report_missing_glyphs`]
/// logs each new one.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MissingGlyphs(pub usize);

/// One sub-rect of a page image the main world has finished painting, to be
/// copied to the GPU after `prepare_assets` ran.
#[derive(Debug, Clone)]
pub(crate) struct QueuedUpload {
    pub image: AssetId<Image>,
    pub x:     u32,
    pub y:     u32,
    pub w:     u32,
    pub h:     u32,
    pub data:  Vec<u8>,
}

/// The uploads waiting on the current frame's extraction. Cleared and refilled
/// by [`update_pages`] so the render world never replays stale rects.
#[derive(Resource, Debug, Default, Clone, ExtractResource)]
pub(crate) struct QueuedUploads(pub(crate) Vec<QueuedUpload>);

/// Applies [`msdf::runtime::DirtyRect`]s to the page images and queues them
/// for the render world.
pub(crate) fn update_pages(
    texts: Query<&MsdfText>,
    default: Option<Res<DefaultFont>>,
    mut images: ResMut<Assets<Image>>,
    mut queue: ResMut<QueuedUploads>,
) {
    queue.0.clear();
    for (font, _) in wanted(&texts, default.as_deref()) {
        let mut state = font.state();
        let dirty = state.atlas.take_dirty();
        if dirty.is_empty() {
            continue;
        }
        while state.pages.len() < state.atlas.page_count() {
            let index = state.pages.len() as u32;
            let image = page_image(&state.atlas, index);
            state.pages.push(images.add(image));
        }
        for rect in dirty {
            let Some(handle) = state.pages.get(rect.page as usize) else {
                continue;
            };
            let Some(image) = images.get_mut_untracked(handle) else {
                continue;
            };
            let Some(page) = state.atlas.page_image(rect.page as usize) else {
                continue;
            };
            let width = image.width();
            if let Some(data) = image.data.as_mut() {
                blit_region(data, width, page, rect);
            }
            let row_bytes = page.width() as usize * 4;
            let mut data = Vec::with_capacity(rect.w as usize * rect.h as usize * 4);
            for row in 0..rect.h {
                let start = (rect.y + row) as usize * row_bytes + rect.x as usize * 4;
                data.extend_from_slice(&page.as_raw()[start..start + rect.w as usize * 4]);
            }
            queue.0.push(QueuedUpload {
                image: handle.id(),
                x: rect.x,
                y: rect.y,
                w: rect.w,
                h: rect.h,
                data,
            });
        }
        drop(state);
    }
}

fn blit_region(data: &mut [u8], image_width: u32, page: &RgbaImage, rect: DirtyRect) {
    let page_row = page.width() as usize * 4;
    let image_row = image_width as usize * 4;
    for row in 0..rect.h {
        let start = (rect.y + row) as usize * page_row + rect.x as usize * 4;
        let src = &page.as_raw()[start..start + rect.w as usize * 4];
        let dst = (rect.y + row) as usize * image_row + rect.x as usize * 4;
        data[dst..dst + src.len()].copy_from_slice(src);
    }
}

fn settings(style: &MsdfStyle, unit_range: Vec2) -> MsdfSettings {
    let outline = style.outline.unwrap_or(Outline {
        color: Color::NONE,
        width: 0.0,
    });
    MsdfSettings {
        color:         LinearRgba::from(style.color).to_vec4(),
        outline_color: LinearRgba::from(outline.color).to_vec4(),
        unit_range,
        outline_width: outline.width.max(0.0),
        emissive:      style.emissive.max(0.0),
    }
}

/// The font a text draws with, if any. `None` means [`DefaultFont`] has not
/// been registered yet and the text has to wait.
fn resolve_font(text: &MsdfText, default: Option<&DefaultFont>) -> Option<Arc<MsdfFont>> {
    text.font.clone().or_else(|| default.map(|default| Arc::clone(&default.0)))
}

/// The union of characters the live texts ask for, grouped by font.
fn wanted(
    texts: &Query<&MsdfText>,
    default: Option<&DefaultFont>,
) -> Vec<(Arc<MsdfFont>, Vec<char>)> {
    let default_font = default.map(|default| Arc::clone(&default.0));
    let mut map: Vec<(Arc<MsdfFont>, Vec<char>)> = Vec::new();
    for text in texts {
        let Some(font) = text.font.clone().or_else(|| default_font.clone()) else {
            continue;
        };
        if let Some((_, chars)) = map.iter_mut().find(|(other, _)| Arc::ptr_eq(other, &font)) {
            chars.extend(text.value.chars());
        } else {
            map.push((font, text.value.chars().collect()));
        }
    }
    map
}

/// Queues glyphs the live texts lack and pins every resident glyph a mesh is
/// drawing against eviction.
pub(crate) fn sync_fonts(texts: Query<&MsdfText>, default: Option<Res<DefaultFont>>) {
    for (font, chars) in wanted(&texts, default.as_deref()) {
        font.sync(&chars);
    }
}

/// Generates glyphs whose requests were queued, so text lands a frame after it
/// first asks.
pub(crate) fn generate_glyphs(texts: Query<&MsdfText>, default: Option<Res<DefaultFont>>) {
    for (font, _) in wanted(&texts, default.as_deref()) {
        let mut state = font.state();
        while let Some(job) = state.atlas.next_job() {
            let rendered = render_glyph(&state.atlas, &job);
            state.atlas.commit(job.ch, &rendered);
        }
        drop(state);
    }
}

/// Lays out every text whose inputs changed or whose missing glyphs landed,
/// and swaps its page children.
pub(crate) fn rebuild_text(
    mut texts: Query<(Entity, &MsdfText, &MsdfStyle, Option<&mut TextRuntime>)>,
    default: Option<Res<DefaultFont>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MsdfMaterial>>,
    mut commands: Commands,
) {
    for (entity, text, style, runtime) in &mut texts {
        let Some(font) = resolve_font(text, default.as_deref()) else {
            continue;
        };

        let (changed, pending, missing, pages) = runtime.as_ref().map_or_else(
            || (true, false, Vec::new(), Vec::new()),
            |runtime| {
                (
                    layout_key(text, &runtime.font) != runtime.layout_key,
                    runtime.pending,
                    runtime.missing.clone(),
                    runtime.pages.clone(),
                )
            },
        );
        let landed = pending
            && missing
                .iter()
                .any(|ch| font.state().atlas.resident(*ch));
        if !changed && !landed {
            continue;
        }

        let value = layout(
            &text.value,
            &font.state().atlas,
            &LayoutOpts {
                size: text.size,
                wrap: text.wrap,
                align: text.align,
                line_height: text.line_height,
                ..Default::default()
            },
        );
        let laid = match value {
            Ok(laid) => laid,
            Err(err) => {
                error!("{entity}: {err}");
                continue;
            }
        };

        let mut children = Vec::new();
        {
            let state = font.state();
            for (page, mesh) in page_meshes(&laid, text.anchor) {
                let Some(handle) = state.pages.get(page as usize) else {
                    continue;
                };
                let material = materials.add(MsdfMaterial {
                    settings: settings(style, state.unit_range),
                    field:    handle.clone(),
                });
                children.push(
                    commands
                        .spawn((
                            Mesh3d(meshes.add(mesh)),
                            MeshMaterial3d(material),
                            NotShadowCaster,
                            Transform::default(),
                            Visibility::default(),
                        ))
                        .id(),
                );
            }
            drop(state);
        }

        for child in &pages {
            commands.entity(*child).despawn();
        }
        let new_runtime = TextRuntime {
            font:       Arc::clone(&font),
            pending:    !laid.missing.is_empty(),
            missing:    laid.missing.clone(),
            layout_key: layout_key(text, &font),
            pages:      children.clone(),
        };
        match runtime {
            Some(mut runtime) => *runtime = new_runtime,
            None => {
                commands.entity(entity).insert(new_runtime);
            }
        }
        if !children.is_empty() {
            commands.entity(entity).add_children(&children);
        }
        commands.entity(entity).insert(MissingGlyphs(laid.missing.len()));
    }
}

/// What a layout depends on, so identical inputs skip a rebuild even while
/// change detection still reports the component.
fn layout_key(text: &MsdfText, font: &MsdfFont) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.value.hash(&mut hasher);
    text.size.to_bits().hash(&mut hasher);
    text.align.hash(&mut hasher);
    text.anchor.hash(&mut hasher);
    text.wrap.map(f32::to_bits).hash(&mut hasher);
    text.line_height.to_bits().hash(&mut hasher);
    (std::ptr::addr_of!(*font) as usize).hash(&mut hasher);
    hasher.finish()
}

/// Restyles without rebuilding the mesh, so a colour fading every frame is
/// cheap.
pub(crate) fn restyle_text(
    changed: Query<(&TextRuntime, &MsdfStyle), Changed<MsdfStyle>>,
    page_materials: Query<&MeshMaterial3d<MsdfMaterial>>,
    mut materials: ResMut<Assets<MsdfMaterial>>,
) {
    for (runtime, style) in &changed {
        let unit_range = runtime.font.state().unit_range;
        let settings = settings(style, unit_range);
        for child in &runtime.pages {
            let Ok(handle) = page_materials.get(*child) else {
                continue;
            };
            if let Some(mut material) = materials.get_mut(handle) {
                material.settings = settings;
            }
        }
    }
}

pub(crate) fn report_missing_glyphs(
    changed: Query<(Entity, &MsdfText, &MissingGlyphs), Changed<MissingGlyphs>>,
) {
    for (entity, text, missing) in &changed {
        if missing.0 > 0 {
            error!(
                "{entity}: {} of {:?} have no glyph in this font and were dropped",
                missing.0, text.value,
            );
        }
    }
}

/// Copies queued sub-rects from CPU pages to the GPU after preparation, so a
/// grown page shows its new glyphs without re-uploading the whole texture.
pub(crate) fn upload_pages(
    queue: Res<QueuedUploads>,
    images: Res<RenderAssets<GpuImage>>,
    queue_wgpu: Res<RenderQueue>,
) {
    for upload in &queue.0 {
        let Some(gpu) = images.get(upload.image) else {
            continue;
        };
        queue_wgpu.0.write_texture(
            TexelCopyTextureInfo {
                texture:   &gpu.texture,
                mip_level: 0,
                origin:    Origin3d {
                    x: upload.x,
                    y: upload.y,
                    z: 0,
                },
                aspect:    TextureAspect::All,
            },
            &upload.data,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(upload.w * 4),
                rows_per_image: Some(upload.h),
            },
            Extent3d {
                width: upload.w,
                height: upload.h,
                depth_or_array_layers: 1,
            },
        );
    }
}
