//! Turns [`MsdfText`] into one mesh per `(font, page)`.
//!
//! The parent entity carries the style and transform; each page it draws
//! becomes a child mesh so every page samples the image that holds it. Glyphs
//! the atlas still has to generate are requested first; one that has not landed
//! yet holds its width open rather than drawing a placeholder that would flash.
//!
//! Nothing here walks every text every frame: [`sync_fonts`] and
//! [`rebuild_text`] both gate on Bevy's own change detection, so an idle scene
//! full of labels costs a handful of cheap "did anything change" checks
//! rather than a re-layout.

use std::{
    collections::HashSet,
    sync::Arc,
};

use bevy::{
    asset::AssetId,
    image::Image,
    light::NotShadowCaster,
    platform::collections::HashMap,
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
use msdf::layout::{
    Align,
    Layout,
    LayoutOpts,
    layout,
};
use smol_str::SmolStr;

use crate::{
    font::{
        DefaultFontStack,
        FontStack,
        MsdfFont,
        asset::FontRequest,
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

/// One child mesh sampling one `(font, page)` of the text's layout, kept
/// across a rebuild so an unrelated glyph landing does not churn every page's
/// mesh and material.
#[derive(Debug)]
struct PageChild {
    entity:   Entity,
    mesh:     Handle<Mesh>,
    material: Handle<MsdfMaterial>,
}

/// What the mesh for a text currently draws. Rebuilt in place whenever the
/// layout inputs or the atlas change; a missing build means the text was just
/// added.
#[derive(Component, Debug, Default)]
pub(crate) struct TextBuild {
    /// True while some of the text's glyphs are still pending.
    pending: bool,
    /// The characters that were missing at the last layout.
    missing: Vec<char>,
    /// One entry per page the text currently draws, keyed by `(font, page)`
    /// so a rebuild can tell which pages are the same page as before.
    pages:   HashMap<(u32, u32), PageChild>,
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
    let mut fonts = default.map_or_default(|default| default.0.clone());
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
        let stack = FontStack::new(stack);
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

/// What [`sync_fonts`] computed last time anything changed, kept so a frame
/// with no text change costs a handful of "did anything change" checks
/// instead of a walk of every character of every text.
#[derive(Default)]
pub(crate) struct SyncFontsCache {
    fonts:  Vec<Arc<MsdfFont>>,
    wanted: Vec<(Arc<MsdfFont>, Vec<char>)>,
    ready:  bool,
}

/// Queues glyphs the live texts lack, pins every resident glyph a mesh is
/// drawing against eviction, and pumps generation for what was queued. Runs
/// before [`rebuild_text`], so a string whose glyphs fit the frame's
/// generation budget draws the frame it appears.
///
/// The character scan that drives this only reruns when some text changed,
/// was removed, or the default stack grew; otherwise the last computed
/// per-font want list is reused. Pumping itself still runs every frame for
/// every known font, since generation is asynchronous and may land several
/// frames after it was queued.
pub(crate) fn sync_fonts(
    texts: Query<&MsdfText>,
    changed: Query<(), Changed<MsdfText>>,
    mut removed: RemovedComponents<MsdfText>,
    default: Option<Res<DefaultFontStack>>,
    time: Res<Time>,
    mut cache: Local<SyncFontsCache>,
) {
    let stack_changed = default.as_ref().is_some_and(DetectChanges::is_changed);
    let dirty =
        !cache.ready || !changed.is_empty() || stack_changed || removed.read().next().is_some();

    if dirty {
        cache.fonts = fonts(&texts, default.as_deref());
        cache.wanted = wanted(&texts, default.as_deref());
        cache.ready = true;
    }

    for font in &cache.fonts {
        let chars = cache
            .wanted
            .iter()
            .find(|(other, _)| Arc::ptr_eq(other, font))
            .map_or(&[][..], |(_, chars)| chars.as_slice());
        font.sync(chars);
        font.tick(time.delta_secs());
        font.pump();
    }
}

/// Lays out every text whose inputs changed, whose stack changed, or whose
/// missing glyphs landed, and reuses its existing page meshes and materials
/// where the rebuild still wants the same `(font, page)`.
pub(crate) fn rebuild_text(
    mut texts: Query<(Entity, Ref<MsdfText>, &MsdfStyle, Option<&mut TextBuild>)>,
    default: Option<Res<DefaultFontStack>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MsdfMaterial>>,
    mut commands: Commands,
) {
    let stack_changed = default.as_ref().is_some_and(DetectChanges::is_changed);

    for (entity, text, style, build) in &mut texts {
        let relayout = text.is_changed() || (text.font.is_none() && stack_changed);
        let stack = match &build {
            None => {
                let Some(stack) = resolve_stack(&text, default.as_deref()) else {
                    continue;
                };
                stack
            }
            Some(build) if relayout || build.pending => {
                let Some(stack) = resolve_stack(&text, default.as_deref()) else {
                    continue;
                };
                // Only pending, not a real input change; skip the rebuild
                // unless something it was waiting on actually landed.
                if !relayout
                    && !build
                        .missing
                        .iter()
                        .any(|ch| stack.iter().any(|font| font.resident(*ch)))
                {
                    continue;
                }
                stack
            }
            Some(_) => continue,
        };

        let source = FontStack::new(stack.clone());
        let laid = layout(
            &text.value,
            &source,
            &LayoutOpts {
                size:        text.size,
                wrap:        text.wrap,
                align:       text.align,
                line_height: text.line_height,
            },
        );
        let unrenderable = laid
            .missing
            .iter()
            .filter(|ch| !source.can_render(**ch))
            .count();

        let mut pages = build.map_or_else(HashMap::default, |mut build| {
            std::mem::take(&mut build.pages)
        });
        reconcile_pages(
            &laid,
            text.anchor,
            style,
            &stack,
            &mut pages,
            &mut meshes,
            &mut materials,
            &mut commands,
            entity,
        );

        let children = pages.values().map(|child| child.entity).collect::<Vec<_>>();
        commands.entity(entity).insert(TextBuild {
            pending: laid.missing.len() > unrenderable,
            missing: laid.missing,
            pages,
        });
        if !children.is_empty() {
            commands.entity(entity).add_children(&children);
        }
        commands.entity(entity).insert(MissingGlyphs(unrenderable));
        if laid.truncated {
            warn!("{entity}: text was truncated to fit the layout cap");
        }
    }
}

/// Reconciles `pages` against what the current layout draws: existing
/// `(font, page)` entries have their mesh updated and their material's
/// settings refreshed in place, new ones spawn, and ones the layout no
/// longer draws despawn.
#[expect(clippy::too_many_arguments)]
fn reconcile_pages(
    laid: &Layout,
    anchor: Anchor,
    style: &MsdfStyle,
    stack: &[Arc<MsdfFont>],
    pages: &mut HashMap<(u32, u32), PageChild>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<MsdfMaterial>,
    commands: &mut Commands,
    parent: Entity,
) {
    let mut drawn = HashSet::new();
    for (key @ (font_index, page), mesh) in page_meshes(laid, anchor) {
        let Some(font) = stack.get(font_index as usize) else {
            continue;
        };
        let Some(handle) = font.page(page) else {
            continue;
        };
        drawn.insert(key);

        if let Some(child) = pages.get(&key) {
            let _ = meshes.insert(&child.mesh, mesh);
            if let Some(mut material) = materials.get_mut(&child.material) {
                let unit_range = material.settings.unit_range;
                material.settings = settings(style, unit_range);
                material.field = handle;
            }
            continue;
        }

        let unit_range = font.unit_range();
        let material = materials.add(MsdfMaterial {
            settings: settings(style, unit_range),
            field:    handle,
        });
        let mesh_handle = meshes.add(mesh);
        let entity = commands
            .spawn((
                Mesh3d(mesh_handle.clone()),
                MeshMaterial3d(material.clone()),
                NotShadowCaster,
                Transform::default(),
                Visibility::default(),
                ChildOf(parent),
            ))
            .id();
        pages.insert(
            key,
            PageChild {
                entity,
                mesh: mesh_handle,
                material,
            },
        );
    }

    pages.retain(|key, child| {
        if drawn.contains(key) {
            return true;
        }
        commands.entity(child.entity).despawn();
        false
    });
}

/// Cleans up the page meshes and [`TextBuild`] of an entity whose
/// [`MsdfText`] was removed without despawning the entity itself (for example
/// clearing an attribute), so a text that comes and goes does not leak page
/// entities.
pub(crate) fn despawn_orphaned_builds(
    mut removed: RemovedComponents<MsdfText>,
    builds: Query<&TextBuild, Without<MsdfText>>,
    mut commands: Commands,
) {
    for entity in removed.read() {
        let Ok(build) = builds.get(entity) else {
            continue;
        };
        for child in build.pages.values() {
            commands.entity(child.entity).despawn();
        }
        commands.entity(entity).remove::<TextBuild>();
    }
}

/// Restyles without rebuilding the mesh, so a colour fading every frame is
/// cheap.
pub(crate) fn restyle_text(
    changed: Query<(&TextBuild, &MsdfStyle), Changed<MsdfStyle>>,
    mut materials: ResMut<Assets<MsdfMaterial>>,
) {
    for (build, style) in &changed {
        for child in build.pages.values() {
            if let Some(mut material) = materials.get_mut(&child.material) {
                let unit_range = material.settings.unit_range;
                material.settings = settings(style, unit_range);
            }
        }
    }
}

/// Characters a registered face does not cover, logged once per change at
/// most and truncated: the text itself is peer-controlled and otherwise an
/// unbounded string lands in the log every time a face registers.
const LOG_PREVIEW: usize = 32;

/// A face still arriving is not a face that lacks the character, so nothing is
/// called tofu until the chain is whole. Registering one re-lays-out every
/// text, which reports again against what actually landed.
pub(crate) fn report_missing_glyphs(
    changed: Query<(Entity, &MsdfText, &MissingGlyphs), Changed<MissingGlyphs>>,
    loading: Query<(), With<FontRequest>>,
) {
    if !loading.is_empty() {
        return;
    }
    for (entity, text, missing) in &changed {
        if missing.0 == 0 {
            continue;
        }
        let preview = text.value.chars().take(LOG_PREVIEW).collect::<String>();
        let elided = if text.value.chars().count() > LOG_PREVIEW {
            "…"
        } else {
            ""
        };
        warn!(
            "{entity}: {} character(s) of {preview:?}{elided} have no glyph in any registered \
             font and draw as tofu",
            missing.0,
        );
    }
}

/// Grows each font's page images and queues the sub-rects that changed since
/// the last call, so the render world copies only what actually moved.
pub(crate) fn update_pages(
    texts: Query<&MsdfText>,
    default: Option<Res<DefaultFontStack>>,
    mut images: ResMut<Assets<Image>>,
    mut queue: ResMut<QueuedUploads>,
) {
    queue.0.clear();
    for font in fonts(&texts, default.as_deref()) {
        for upload in font.drain_dirty(&mut images) {
            queue.0.push(QueuedUpload {
                image: upload.image,
                x:     upload.rect.x,
                y:     upload.rect.y,
                w:     upload.rect.w,
                h:     upload.rect.h,
                data:  upload.data,
            });
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
        atlas::AtlasOpts,
        font::Font,
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
                    despawn_orphaned_builds,
                    report_missing_glyphs,
                )
                    .chain(),
            );

        let font = Font::parse(Arc::<[u8]>::from(notosans::REGULAR_TTF)).expect("parse");
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        let font = MsdfFont::new(Arc::new(font), AtlasOpts::default(), &mut images);
        app.insert_resource(DefaultFontStack(vec![Arc::new(font)]));
        app
    }

    fn spawn(app: &mut App, text: MsdfText) -> Entity {
        app.world_mut().spawn(text).id()
    }

    /// Ticks until the glyphs an async pump needs to land have landed, or
    /// panics: generation runs off-thread now, so a test cannot assume one
    /// `app.update()` is enough.
    fn settle(app: &mut App, entity: Entity) {
        for _ in 0..500 {
            app.update();
            let done = app
                .world()
                .get::<TextBuild>(entity)
                .is_some_and(|build| !build.pending);
            if done {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        panic!("text never finished generating");
    }

    #[test]
    fn a_string_lands_as_page_children_once_generation_catches_up() {
        let mut app = app();
        let entity = spawn(
            &mut app,
            MsdfText {
                value: SmolStr::new("hello"),
                ..Default::default()
            },
        );
        settle(&mut app, entity);

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
    fn an_unchanged_text_is_not_rebuilt_on_a_later_frame() {
        let mut app = app();
        let entity = spawn(
            &mut app,
            MsdfText {
                value: SmolStr::new("hello"),
                ..Default::default()
            },
        );
        settle(&mut app, entity);
        let pages_before = app
            .world()
            .get::<TextBuild>(entity)
            .expect("build")
            .pages
            .values()
            .map(|child| (child.entity, child.mesh.clone()))
            .collect::<Vec<_>>();

        app.update();
        app.update();

        let pages_after = app
            .world()
            .get::<TextBuild>(entity)
            .expect("build")
            .pages
            .values()
            .map(|child| (child.entity, child.mesh.clone()))
            .collect::<Vec<_>>();
        assert_eq!(
            pages_before, pages_after,
            "the same page entities and mesh handles survive frames with no change"
        );
    }

    #[test]
    fn changing_the_value_rebuilds_but_reuses_the_page_entity() {
        let mut app = app();
        let entity = spawn(
            &mut app,
            MsdfText {
                value: SmolStr::new("hi"),
                ..Default::default()
            },
        );
        settle(&mut app, entity);
        let before = app
            .world()
            .get::<TextBuild>(entity)
            .expect("build")
            .pages
            .values()
            .map(|child| child.entity)
            .collect::<Vec<_>>();

        app.world_mut()
            .get_mut::<MsdfText>(entity)
            .expect("text")
            .value = SmolStr::new("ho");
        settle(&mut app, entity);
        let after = app
            .world()
            .get::<TextBuild>(entity)
            .expect("build")
            .pages
            .values()
            .map(|child| child.entity)
            .collect::<Vec<_>>();
        assert_eq!(
            before, after,
            "the same font and page reuse the page's mesh and material"
        );
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
        settle(&mut app, entity);

        assert_eq!(
            app.world().get::<MissingGlyphs>(entity).copied(),
            Some(MissingGlyphs(1)),
            "the same missing character is counted once"
        );
        let build = app.world().get::<TextBuild>(entity).expect("build");
        assert!(
            !build.pending,
            "nothing is coming for it, so the text stops waiting"
        );
    }

    #[test]
    fn a_string_past_the_glyph_cap_is_truncated_rather_than_erroring() {
        let mut app = app();
        let entity = spawn(
            &mut app,
            MsdfText {
                value: SmolStr::new("a".repeat(msdf::layout::MAX_GLYPHS + 1).as_str()),
                ..Default::default()
            },
        );
        settle(&mut app, entity);

        let build = app.world().get::<TextBuild>(entity).expect("build");
        assert!(!build.pages.is_empty(), "the truncated text still draws");
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
        assert!(app.world().get::<TextBuild>(entity).is_none());
    }

    #[test]
    fn removing_the_text_component_despawns_its_pages() {
        let mut app = app();
        let entity = spawn(
            &mut app,
            MsdfText {
                value: SmolStr::new("hi"),
                ..Default::default()
            },
        );
        settle(&mut app, entity);
        let child = app
            .world()
            .get::<Children>(entity)
            .expect("children")
            .iter()
            .next()
            .expect("one child");

        app.world_mut().entity_mut(entity).remove::<MsdfText>();
        app.update();

        assert!(app.world().get::<TextBuild>(entity).is_none());
        assert!(
            app.world().get_entity(child).is_err(),
            "the orphaned page mesh is despawned, not leaked"
        );
    }
}
