//! The assets every UNAVI app pulls over iroh, each defined once, and the
//! plugin that serves them.
//!
//! [`AssetsPlugin`] registers the `iroh://` asset source and requests the
//! fallback font stack.

use bevy::prelude::*;
use bevy_iroh::assets::{
    AssetSpec,
    IrohAssetsPlugin,
    hex_hash,
};
use bevy_msdf::font::asset::{
    FontBytes,
    FontFace,
};

/// Every asset must be hosted by a reachable unavi-server (or other provider).
pub const DEFAULT_AVATAR: AssetSpec = AssetSpec {
    rel_path: "model/default.vrm",
    hash:     hex_hash("a2f1a48db6cdf369ab510f6a6fb869d107897231b70c4920ad0357e4930c6281"),
    size:     4_452_486,
};

pub const DEFAULT_CHARACTER_ANIMATIONS: AssetSpec = AssetSpec {
    rel_path: "model/animations.glb",
    hash:     hex_hash("9fbda809b00ab14e58356721e0c0a92fe88b9234c486a43b9417c4f27555c0c6"),
    size:     506_152,
};

/// Noto Sans Regular, SIL Open Font License 1.1. Latin, Greek and Cyrillic.
pub const DEFAULT_FONT: AssetSpec = AssetSpec {
    rel_path: "font/noto-sans.ttf",
    hash:     hex_hash("3a21ac778bcc91b57dc32576c6baffbcb493b78b4b6ad46b05c3d33bb5da7315"),
    size:     621_572,
};

/// Noto Sans CJK JP Regular, SIL Open Font License 1.1. Covers the Han,
/// hangul and kana [`DEFAULT_FONT`] lacks, in one face rather than one per
/// language.
pub const CJK_FONT: AssetSpec = AssetSpec {
    rel_path: "font/noto-sans-cjk.otf",
    hash:     hex_hash("1580ba0d54c84191041a55ec8d442d5a7d3668e5af8c9fee5456c776c30ff16a"),
    size:     16_467_736,
};

/// The assets the client pulls over iroh.
pub const MANIFEST: &[AssetSpec] = &[
    DEFAULT_AVATAR,
    DEFAULT_CHARACTER_ANIMATIONS,
    DEFAULT_FONT,
    CJK_FONT,
];

/// The fallback chain text draws with, primary first. A character resolves to
/// the first face here that covers it.
pub const FONT_STACK: &[AssetSpec] = &[DEFAULT_FONT, CJK_FONT];

/// The path Bevy loads `spec` by, on every platform.
#[must_use]
pub fn path(spec: &AssetSpec) -> String {
    bevy_iroh::assets::asset_path(spec.rel_path)
}

/// Serves [`MANIFEST`] over the `iroh://` asset source and loads
/// [`FONT_STACK`].
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        // Must exist before `AssetPlugin` builds the sources it knows about.
        app.add_plugins(IrohAssetsPlugin::new(MANIFEST))
            .add_systems(Startup, load_font_stack);
    }
}

/// Requests the fallback chain over iroh. No face is embedded, so text draws
/// nothing until the primary arrives.
fn load_font_stack(mut commands: Commands, assets: Res<AssetServer>) {
    for (order, spec) in FONT_STACK.iter().enumerate() {
        commands.spawn((
            Name::new(format!("font {}", spec.rel_path)),
            FontFace::new(assets.load::<FontBytes>(path(spec)), order as u32),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_size_is_recorded() {
        for asset in MANIFEST {
            assert!(asset.size > 0, "{} has no size", asset.rel_path);
        }
    }
}
