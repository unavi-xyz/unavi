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
use bevy_msdf::{
    font::asset::{
        FontBytes,
        FontRequest,
    },
    text::MsdfText,
};

/// Every asset must be hosted by a reachable unavi-node (or other provider).
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

/// The assets the app pulls over iroh.
pub const MANIFEST: &[AssetSpec] = &[
    DEFAULT_AVATAR,
    DEFAULT_CHARACTER_ANIMATIONS,
    DEFAULT_FONT,
    CJK_FONT,
];

/// The fallback chain text draws with, primary first. A character resolves to
/// the first face here that covers it. The primary loads at startup and the
/// rest on demand.
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
            .add_systems(Startup, load_primary_font)
            .add_systems(
                Update,
                request_cjk_font.run_if(not(resource_exists::<CjkRequested>)),
            );
    }
}

/// Present once the CJK face has been requested, which ends the text scan.
#[derive(Resource)]
struct CjkRequested;

/// Whether `c` needs [`CJK_FONT`]. Covers Han (with radicals, compatibility
/// ideographs and the supplementary planes), kana, bopomofo, hangul, CJK
/// punctuation, enclosed forms and the half- and full-width forms.
const fn is_cjk(c: char) -> bool {
    matches!(
        c,
        '\u{1100}'..='\u{11FF}'
            | '\u{2E80}'..='\u{2FDF}'
            | '\u{3000}'..='\u{33FF}'
            | '\u{3400}'..='\u{4DBF}'
            | '\u{4E00}'..='\u{9FFF}'
            | '\u{AC00}'..='\u{D7AF}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{FF00}'..='\u{FFEF}'
            | '\u{20000}'..='\u{3134F}'
    )
}

/// Requests `spec` at its position in [`FONT_STACK`], or warns if it is not
/// part of the stack.
fn request_font(commands: &mut Commands, assets: &AssetServer, spec: &AssetSpec) {
    let Some(order) = FONT_STACK
        .iter()
        .position(|face| face.rel_path == spec.rel_path)
    else {
        warn!(path = spec.rel_path, "font is not in the fallback stack");
        return;
    };
    commands.spawn((
        Name::new(format!("font {}", spec.rel_path)),
        FontRequest::new(assets.load::<FontBytes>(path(spec)), order as u32),
    ));
}

/// Requests the primary face over iroh. No face is embedded, so text draws
/// nothing until it arrives.
fn load_primary_font(mut commands: Commands, assets: Res<AssetServer>) {
    request_font(&mut commands, &assets, &DEFAULT_FONT);
}

/// Requests the CJK face the first time changed text contains a character
/// only it covers. It keeps its place in [`FONT_STACK`], so the chain order
/// does not depend on when it is requested.
fn request_cjk_font(
    mut commands: Commands,
    assets: Res<AssetServer>,
    texts: Query<&MsdfText, Changed<MsdfText>>,
) {
    if texts.iter().any(|text| text.value.chars().any(is_cjk)) {
        request_font(&mut commands, &assets, &CJK_FONT);
        commands.insert_resource(CjkRequested);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cjk_blocks_are_detected() {
        for c in ["漢", "ぁ", "カ", "한", "ᄀ", "、", "Ａ", "ｱ", "㐀"] {
            assert!(c.chars().all(is_cjk), "{c} should need the CJK face");
        }
    }

    #[test]
    fn other_scripts_are_not_cjk() {
        for c in ["a", "Z", "é", "Ω", "Я", " ", "😀", "!"] {
            assert!(!c.chars().any(is_cjk), "{c} should not need the CJK face");
        }
    }

    #[test]
    fn the_cjk_face_is_second_in_the_stack() {
        assert_eq!(FONT_STACK[1].rel_path, CJK_FONT.rel_path);
    }

    #[test]
    fn a_size_is_recorded() {
        for asset in MANIFEST {
            assert!(asset.size > 0, "{} has no size", asset.rel_path);
        }
    }
}
