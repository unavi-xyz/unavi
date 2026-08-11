//! World-space text from a multi-channel distance field.
//!
//! Bevy's own text is 2D: it extracts into the sprite pipeline and needs a 2D
//! camera, and its glyphs are rasterized per font size. Neither survives a
//! placard read at 0.4 m and a sign read at 20 m from the same asset, which is
//! what a headset asks for.
//!
//! The field is baked from Noto Sans (SIL Open Font License 1.1) by this
//! crate's build script.

use bevy::{
    asset::embedded_asset,
    prelude::*,
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
        ))
        .init_asset::<font::MsdfFont>()
        .add_systems(Startup, font::register_default_font)
        .add_systems(
            Update,
            (
                text::rebuild_text,
                text::restyle_text,
                text::report_missing_glyphs,
            )
                .chain()
                .in_set(MsdfSet),
        );
    }
}
