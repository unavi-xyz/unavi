//! World-space text from a multi-channel distance field.
//!
//! Bevy's own text is 2D, needs a 2D camera, and rasterizes per font size; a
//! field keeps one asset sharp at any distance.
//!
//! The field is grown at runtime from Noto Sans (SIL Open Font License 1.1) by
//! a closed-budget atlas, so any script the bundled face covers renders
//! without a per-charset bake step.

use bevy::{
    asset::embedded_asset,
    prelude::*,
    render::{
        Render,
        RenderApp,
        extract_resource::ExtractResourcePlugin,
        render_asset::prepare_assets,
        texture::GpuImage,
    },
};

pub mod billboard;
pub mod font;
pub mod material;
pub mod mesh;
pub mod text;

/// Text is rebuilt here, so a caller writing to [`text::MsdfText`] should run
/// before it to have the change drawn the same frame.
#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MsdfSet;

pub struct MsdfPlugin;

impl Plugin for MsdfPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "msdf.wgsl");

        app.add_plugins((
            MaterialPlugin::<material::MsdfMaterial>::default(),
            billboard::plugin,
            ExtractResourcePlugin::<text::QueuedUploads>::default(),
        ))
        .init_resource::<text::QueuedUploads>()
        .add_systems(Startup, font::register_default_font)
        .add_systems(
            Update,
            (
                text::sync_fonts,
                text::generate_glyphs,
                text::update_pages,
                text::rebuild_text,
                text::restyle_text,
                text::report_missing_glyphs,
            )
                .chain()
                .in_set(MsdfSet),
        );

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(
                Render,
                text::upload_pages
                    .after(prepare_assets::<GpuImage>),
            );
        }
    }
}
