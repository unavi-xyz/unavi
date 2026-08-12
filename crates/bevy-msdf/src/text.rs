//! Turns [`MsdfText`] into one mesh per atlas page.
//!
//! The parent entity carries the style and transform; each page it draws
//! becomes a child mesh so every page samples the image that holds it. Glyphs
//! the atlas still has to generate are requested first; one that has not landed
//! yet holds its width open rather than drawing a placeholder that would flash.

use std::{
    collections::HashSet,
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
    atlas::Rect,
    layout::{
        Align,
        Laid,
        LayoutOpts,
        layout,
    },
    runtime::DirtyRect,
};
use smol_str::SmolStr;

use crate::{
    font::{
        DefaultFontStack,
        FontStack,
        MsdfFont,
        asset::FontFace,
        generate,
        page_image,
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
    /// `None` draws with [`DefaultFontStack`].
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
    /// The fallback chain the text was laid out against, so a change to it
    /// (e.g. a newly registered font) invalidates the layout.
    stack:      Vec<Arc<MsdfFont>>,
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

/// The distance range baked into a child's page, captured at build time so a
/// restyle never re-reads the font's atlas.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct MsdfUnitRange(Vec2);

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

/// Distinct characters no font in the text's stack can draw.
///
/// They render as tofu until a face that covers them is registered.
/// Characters merely waiting on generation are not counted: they land on their
/// own.
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
    default: Option<Res<DefaultFontStack>>,
    mut images: ResMut<Assets<Image>>,
    mut queue: ResMut<QueuedUploads>,
) {
    queue.0.clear();
    for font in fonts(&texts, default.as_deref()) {
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
            let Some(data) = rows(page, rect) else {
                continue;
            };
            if image.width() == page.width()
                && let Some(target) = image.data.as_mut()
            {
                blit_region(target, page.width(), &data, rect);
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

/// The rect's texels, row by row. `None` when the rect is not inside the page,
/// which no atlas produces but no slice should trust either.
fn rows(page: &RgbaImage, rect: DirtyRect) -> Option<Vec<u8>> {
    if rect.x + rect.w > page.width() || rect.y + rect.h > page.height() {
        return None;
    }
    let row_bytes = page.width() as usize * 4;
    let width = rect.w as usize * 4;
    let mut data = Vec::with_capacity(width * rect.h as usize);
    for row in 0..rect.h {
        let start = (rect.y + row) as usize * row_bytes + rect.x as usize * 4;
        data.extend_from_slice(page.as_raw().get(start..start + width)?);
    }
    Some(data)
}

fn blit_region(target: &mut [u8], width: u32, data: &[u8], rect: DirtyRect) {
    let image_row = width as usize * 4;
    let rect_row = rect.w as usize * 4;
    for row in 0..rect.h as usize {
        let start = (rect.y as usize + row) * image_row + rect.x as usize * 4;
        let (Some(dst), Some(src)) = (
            target.get_mut(start..start + rect_row),
            data.get(row * rect_row..(row + 1) * rect_row),
        ) else {
            return;
        };
        dst.copy_from_slice(src);
    }
}

fn settings(style: &MsdfStyle, unit_range: Vec2) -> MsdfSettings {
    let outline = style.outline.unwrap_or(Outline {
        color: Color::NONE,
        width: 0.0,
    });
    MsdfSettings {
        color: LinearRgba::from(style.color).to_vec4(),
        outline_color: LinearRgba::from(outline.color).to_vec4(),
        unit_range,
        outline_width: outline.width.max(0.0),
        emissive: style.emissive.max(0.0),
    }
}

/// The fallback chain a text draws with, if any. `None` means no font is
/// registered yet and the text has to wait.
fn resolve_stack(
    text: &MsdfText,
    default: Option<&DefaultFontStack>,
) -> Option<Vec<Arc<MsdfFont>>> {
    let stack = text.font.clone().map_or_else(
        || default.map(|default| default.0.clone()),
        |font| Some(vec![font]),
    )?;
    (!stack.is_empty()).then_some(stack)
}

/// Every font any live text could draw with, deduplicated. Fonts nothing draws
/// are included: they still hold pins to release and pages to upload.
fn fonts(texts: &Query<&MsdfText>, default: Option<&DefaultFontStack>) -> Vec<Arc<MsdfFont>> {
    let mut fonts = default.map(|default| default.0.clone()).unwrap_or_default();
    for text in texts {
        let Some(font) = &text.font else { continue };
        if !fonts.iter().any(|other| Arc::ptr_eq(other, font)) {
            fonts.push(Arc::clone(font));
        }
    }
    fonts
}

/// The distinct characters the live texts ask for, each assigned to the first
/// font in its text's stack that can serve it.
fn wanted(
    texts: &Query<&MsdfText>,
    default: Option<&DefaultFontStack>,
) -> Vec<(Arc<MsdfFont>, Vec<char>)> {
    let mut map: Vec<(Arc<MsdfFont>, HashSet<char>)> = Vec::new();
    for text in texts {
        let Some(stack) = resolve_stack(text, default) else {
            continue;
        };
        let stack = FontStack::new(stack.clone());
        let mut seen = HashSet::new();
        for ch in text.value.chars() {
            if !seen.insert(ch) {
                continue;
            }
            let Some(font) = stack.serving(ch).and_then(|index| stack.font(index)) else {
                continue;
            };
            match map.iter_mut().find(|(other, _)| Arc::ptr_eq(other, font)) {
                Some((_, chars)) => {
                    chars.insert(ch);
                }
                None => map.push((Arc::clone(font), HashSet::from([ch]))),
            }
        }
    }
    map.into_iter()
        .map(|(font, chars)| (font, chars.into_iter().collect()))
        .collect()
}

/// Queues glyphs the live texts lack, pins every resident glyph a mesh is
/// drawing against eviction, and generates what was queued. Runs before
/// [`rebuild_text`], so a string whose glyphs fit the frame's generation budget
/// draws the frame it appears.
pub(crate) fn sync_fonts(
    texts: Query<&MsdfText>,
    default: Option<Res<DefaultFontStack>>,
    time: Res<Time>,
) {
    let wanted = wanted(&texts, default.as_deref());
    for font in fonts(&texts, default.as_deref()) {
        let chars = wanted
            .iter()
            .find(|(other, _)| Arc::ptr_eq(other, &font))
            .map(|(_, chars)| chars.as_slice())
            .unwrap_or_default();
        font.sync(chars);

        let mut state = font.state();
        state.atlas.advance(time.delta_secs());
        generate(&mut state.atlas);
        drop(state);
    }
}

/// Lays out every text whose inputs changed or whose missing glyphs landed,
/// and swaps its page children.
pub(crate) fn rebuild_text(
    mut texts: Query<(Entity, &MsdfText, &MsdfStyle, Option<&mut TextRuntime>)>,
    default: Option<Res<DefaultFontStack>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MsdfMaterial>>,
    mut commands: Commands,
) {
    for (entity, text, style, runtime) in &mut texts {
        let Some(stack) = resolve_stack(text, default.as_deref()) else {
            continue;
        };

        let (changed, pending, missing, pages) = runtime.as_ref().map_or_else(
            || (true, false, Vec::new(), Vec::new()),
            |runtime| {
                (
                    layout_key(text, &runtime.stack) != runtime.layout_key,
                    runtime.pending,
                    runtime.missing.clone(),
                    runtime.pages.clone(),
                )
            },
        );
        let landed = pending
            && missing
                .iter()
                .any(|ch| stack.iter().any(|font| font.state().atlas.resident(*ch)));
        if !changed && !landed {
            continue;
        }

        let source = FontStack::new(stack.clone());
        let value = layout(
            &text.value,
            &source,
            &LayoutOpts {
                size: text.size,
                wrap: text.wrap,
                align: text.align,
                line_height: text.line_height,
                ..Default::default()
            },
        );
        // A layout that cannot be built is still recorded, so the same string
        // is not retried — and its error re-logged — every frame.
        let laid = match value {
            Ok(laid) => laid,
            Err(err) => {
                error!("{entity}: {err}");
                Laid {
                    quads:   Vec::new(),
                    bounds:  Rect::ZERO,
                    ink:     Rect::ZERO,
                    lines:   0,
                    missing: Vec::new(),
                }
            }
        };
        let unrenderable = laid
            .missing
            .iter()
            .filter(|ch| !source.can_render(**ch))
            .count();

        let children = spawn_pages(
            &laid,
            text,
            style,
            &stack,
            &mut meshes,
            &mut materials,
            &mut commands,
        );

        for child in &pages {
            commands.entity(*child).despawn();
        }
        let new_runtime = TextRuntime {
            stack:      stack.clone(),
            pending:    laid.missing.len() > unrenderable,
            missing:    laid.missing.clone(),
            layout_key: layout_key(text, &stack),
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
        commands.entity(entity).insert(MissingGlyphs(unrenderable));
    }
}

/// One child per `(font, page)` the laid-out text draws, each sampling the
/// image that holds its glyphs.
fn spawn_pages(
    laid: &Laid,
    text: &MsdfText,
    style: &MsdfStyle,
    stack: &[Arc<MsdfFont>],
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<MsdfMaterial>,
    commands: &mut Commands,
) -> Vec<Entity> {
    let mut children = Vec::new();
    for ((font_index, page), mesh) in page_meshes(laid, text.anchor) {
        let Some(font) = stack.get(font_index as usize) else {
            continue;
        };
        let state = font.state();
        let handle = state.pages.get(page as usize).cloned();
        let unit_range = state.unit_range;
        drop(state);
        let Some(handle) = handle else { continue };
        let material = materials.add(MsdfMaterial {
            settings: settings(style, unit_range),
            field:    handle,
        });
        children.push(
            commands
                .spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(material),
                    MsdfUnitRange(unit_range),
                    NotShadowCaster,
                    Transform::default(),
                    Visibility::default(),
                ))
                .id(),
        );
    }
    children
}

/// What a layout depends on, so identical inputs skip a rebuild even while
/// change detection still reports the component.
fn layout_key(text: &MsdfText, stack: &[Arc<MsdfFont>]) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.value.hash(&mut hasher);
    text.size.to_bits().hash(&mut hasher);
    text.align.hash(&mut hasher);
    text.anchor.hash(&mut hasher);
    text.wrap.map(f32::to_bits).hash(&mut hasher);
    text.line_height.to_bits().hash(&mut hasher);
    for font in stack {
        (std::ptr::addr_of!(**font) as usize).hash(&mut hasher);
    }
    hasher.finish()
}

/// Restyles without rebuilding the mesh, so a colour fading every frame is
/// cheap.
pub(crate) fn restyle_text(
    changed: Query<(&TextRuntime, &MsdfStyle), Changed<MsdfStyle>>,
    page_materials: Query<(&MeshMaterial3d<MsdfMaterial>, &MsdfUnitRange)>,
    mut materials: ResMut<Assets<MsdfMaterial>>,
) {
    for (runtime, style) in &changed {
        for child in &runtime.pages {
            let Ok((handle, unit_range)) = page_materials.get(*child) else {
                continue;
            };
            if let Some(mut material) = materials.get_mut(handle) {
                material.settings = settings(style, unit_range.0);
            }
        }
    }
}

/// A face still arriving is not a face that lacks the character, so nothing is
/// called tofu until the chain is whole. Registering one re-lays-out every
/// text, which reports again against what actually landed.
pub(crate) fn report_missing_glyphs(
    changed: Query<(Entity, &MsdfText, &MissingGlyphs), Changed<MissingGlyphs>>,
    loading: Query<(), With<FontFace>>,
) {
    if !loading.is_empty() {
        return;
    }
    for (entity, text, missing) in &changed {
        if missing.0 > 0 {
            warn!(
                "{entity}: {} character(s) of {:?} have no glyph in any registered font and draw \
                 as tofu",
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
                offset:         0,
                bytes_per_row:  Some(upload.w * 4),
                rows_per_image: Some(upload.h),
            },
            Extent3d {
                width:                 upload.w,
                height:                upload.h,
                depth_or_array_layers: 1,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use bevy::asset::AssetPlugin;
    use msdf::{
        font::Font,
        runtime::RuntimeOpts,
    };

    use super::*;
    use crate::font::MsdfFont;

    /// The systems under test, without the render app a full `MsdfPlugin`
    /// would bring.
    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Image>()
            .init_asset::<Mesh>()
            .init_asset::<MsdfMaterial>()
            .init_resource::<QueuedUploads>()
            .add_systems(
                Update,
                (
                    sync_fonts,
                    update_pages,
                    rebuild_text,
                    report_missing_glyphs,
                )
                    .chain(),
            );

        let font = Font::parse(Arc::<[u8]>::from(notosans::REGULAR_TTF)).expect("parse");
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        let font = MsdfFont::new(Arc::new(font), RuntimeOpts::default(), &mut images);
        app.insert_resource(DefaultFontStack(vec![Arc::new(font)]));
        app
    }

    fn spawn(app: &mut App, text: MsdfText) -> Entity {
        app.world_mut().spawn(text).id()
    }

    #[test]
    fn a_string_lands_as_page_children_in_one_frame() {
        let mut app = app();
        let entity = spawn(
            &mut app,
            MsdfText {
                value: SmolStr::new("hello"),
                ..Default::default()
            },
        );
        app.update();

        let world = app.world();
        assert_eq!(
            world.get::<MissingGlyphs>(entity).copied(),
            Some(MissingGlyphs(0)),
            "an unseeded font generates what the text asked for"
        );
        let children = world.get::<Children>(entity).expect("children");
        assert_eq!(children.len(), 1, "one font, one page, one mesh");
        assert!(world.get::<Mesh3d>(children[0]).is_some());
    }

    #[test]
    fn a_character_no_face_covers_is_reported_once_and_still_draws() {
        let mut app = app();
        let entity = spawn(
            &mut app,
            MsdfText {
                value: SmolStr::new("a漢b漢"),
                ..Default::default()
            },
        );
        app.update();

        assert_eq!(
            app.world().get::<MissingGlyphs>(entity).copied(),
            Some(MissingGlyphs(1)),
            "the same missing character is counted once"
        );
        let runtime = app.world().get::<TextRuntime>(entity).expect("runtime");
        assert!(
            !runtime.pending,
            "nothing is coming for it, so the text stops waiting"
        );
    }

    #[test]
    fn a_string_past_the_glyph_cap_is_not_retried_every_frame() {
        let mut app = app();
        let entity = spawn(
            &mut app,
            MsdfText {
                value: SmolStr::new("a".repeat(msdf::layout::MAX_GLYPHS + 1)),
                ..Default::default()
            },
        );
        app.update();

        let runtime = app.world().get::<TextRuntime>(entity).expect("runtime");
        assert!(runtime.pages.is_empty(), "a refused layout draws nothing");
        assert!(!runtime.pending);
        let key = runtime.layout_key;

        app.update();
        assert_eq!(
            app.world()
                .get::<TextRuntime>(entity)
                .expect("runtime")
                .layout_key,
            key,
            "the failed layout is recorded rather than rebuilt every frame"
        );
    }

    #[test]
    fn a_text_with_no_font_registered_waits_rather_than_drawing() {
        let mut app = app();
        app.insert_resource(DefaultFontStack(Vec::new()));
        let entity = spawn(
            &mut app,
            MsdfText {
                value: SmolStr::new("hello"),
                ..Default::default()
            },
        );
        app.update();
        assert!(app.world().get::<TextRuntime>(entity).is_none());
    }

    #[test]
    fn a_rect_outside_its_page_is_dropped_rather_than_sliced() {
        let page = RgbaImage::new(8, 8);
        let rect = |x, y, w, h| DirtyRect {
            page: 0,
            x,
            y,
            w,
            h,
        };
        assert!(rows(&page, rect(0, 0, 8, 8)).is_some());
        assert!(rows(&page, rect(4, 4, 8, 8)).is_none());
        assert!(rows(&page, rect(0, 0, 9, 1)).is_none());
    }

    #[test]
    fn a_blit_into_a_smaller_target_stops_at_its_end() {
        let mut target = vec![0u8; 4 * 4 * 4];
        let data = vec![7u8; 4 * 4];
        blit_region(
            &mut target,
            4,
            &data,
            DirtyRect {
                page: 0,
                x:    2,
                y:    3,
                w:    4,
                h:    4,
            },
        );
        assert!(target.iter().all(|byte| *byte == 0), "nothing overran");
    }
}
