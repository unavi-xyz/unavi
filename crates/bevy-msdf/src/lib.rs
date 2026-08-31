//! World-space text from a multi-channel distance field.
//!
//! Bevy's own text is 2D, needs a 2D camera, and rasterizes per font size; a
//! field keeps one asset sharp at any distance.
//!
//! No face is embedded. A consumer spawns [`font::asset::FontFace`] for each
//! face of its fallback chain and a closed-budget atlas grows a field from
//! whatever arrives; coverage follows from which files are fetched.

// Bevy`s `AsBindGroup` needs higher limit
#![recursion_limit = "256"]

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
        .init_asset::<font::asset::FontBytes>()
        .init_asset_loader::<font::asset::FontBytesLoader>()
        .init_resource::<text::QueuedUploads>()
        .init_resource::<font::DefaultFontStack>()
        .add_observer(font::on_register_font)
        .add_systems(
            Update,
            (
                font::asset::register_loaded_faces
                    .run_if(any_with_component::<font::asset::FontFace>),
                text::sync_fonts,
                text::update_pages,
                text::rebuild_text,
                text::restyle_text,
                text::report_missing_glyphs,
            )
                .chain()
                .in_set(MsdfSet),
        );

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(Render, text::upload_pages.after(prepare_assets::<GpuImage>));
        }
    }
}
