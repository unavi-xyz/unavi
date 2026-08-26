//! The UNAVI manifest over iroh, and the fonts every app draws text with.
//!
//! [`UnaviAssetsPlugin`] registers the `iroh://` asset source and requests the
//! fallback font stack.

use bevy::prelude::*;
use bevy_iroh_assets::IrohAssetsPlugin;
use bevy_msdf::font::asset::{
    FontBytes,
    FontFace,
};
use unavi_assets::{
    FONT_STACK,
    asset_path,
};

/// The assets the client pulls over iroh. Every entry must be hosted by a
/// reachable unavi-server (or other provider).
pub const MANIFEST: &[bevy_iroh_assets::AssetSpec] = &[
    bevy_iroh_assets::AssetSpec {
        rel_path: unavi_assets::DEFAULT_AVATAR,
        hash:     "a2f1a48db6cdf369ab510f6a6fb869d107897231b70c4920ad0357e4930c6281",
    },
    bevy_iroh_assets::AssetSpec {
        rel_path: unavi_assets::DEFAULT_CHARACTER_ANIMATIONS,
        hash:     "9fbda809b00ab14e58356721e0c0a92fe88b9234c486a43b9417c4f27555c0c6",
    },
    bevy_iroh_assets::AssetSpec {
        rel_path: unavi_assets::DEFAULT_FONT,
        hash:     "3a21ac778bcc91b57dc32576c6baffbcb493b78b4b6ad46b05c3d33bb5da7315",
    },
    bevy_iroh_assets::AssetSpec {
        rel_path: unavi_assets::CJK_FONT,
        hash:     "1580ba0d54c84191041a55ec8d442d5a7d3668e5af8c9fee5456c776c30ff16a",
    },
];

pub struct UnaviAssetsPlugin;

impl Plugin for UnaviAssetsPlugin {
    fn build(&self, app: &mut App) {
        // Must exist before `AssetPlugin` builds the sources it knows about.
        app.add_plugins(IrohAssetsPlugin::new(MANIFEST))
            .add_systems(Startup, load_font_stack);
    }
}

/// Requests the fallback chain over iroh. No face is embedded, so text draws
/// nothing until the primary arrives.
fn load_font_stack(mut commands: Commands, assets: Res<AssetServer>) {
    for (order, path) in FONT_STACK.iter().enumerate() {
        commands.spawn((
            Name::new(format!("font {path}")),
            FontFace::new(assets.load::<FontBytes>(asset_path(path)), order as u32),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_named_asset_is_hosted() {
        for rel_path in FONT_STACK.iter().copied().chain([
            unavi_assets::DEFAULT_AVATAR,
            unavi_assets::DEFAULT_CHARACTER_ANIMATIONS,
        ]) {
            assert!(
                MANIFEST.iter().any(|asset| asset.rel_path == rel_path),
                "{rel_path} names no manifest entry, so nothing can serve it"
            );
        }
    }

    #[test]
    fn a_hash_is_a_blake3_digest() {
        for asset in MANIFEST {
            assert_eq!(asset.hash.len(), 64, "{} is not 32 bytes", asset.rel_path);
            assert!(
                asset.hash.chars().all(|ch| ch.is_ascii_hexdigit()),
                "{} is not hex",
                asset.rel_path
            );
        }
    }
}
