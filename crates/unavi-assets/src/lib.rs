pub const DEFAULT_AVATAR: &str = "model/default.vrm";
pub const DEFAULT_CHARACTER_ANIMATIONS: &str = "model/animations.glb";

/// Noto Sans Regular, SIL Open Font License 1.1. Latin, Greek and Cyrillic.
pub const DEFAULT_FONT: &str = "font/noto-sans.ttf";
/// Noto Sans CJK JP Regular, SIL Open Font License 1.1. Covers the Han,
/// hangul and kana [`DEFAULT_FONT`] lacks, in one face rather than one per
/// language.
pub const CJK_FONT: &str = "font/noto-sans-cjk.otf";

/// The fallback chain text draws with, primary first. A character resolves to
/// the first face here that covers it.
pub const FONT_STACK: &[&str] = &[DEFAULT_FONT, CJK_FONT];

/// The path Bevy loads an asset by, on every platform.
#[must_use]
pub fn asset_path(relative_path: &str) -> String {
    format!("iroh://{relative_path}")
}

#[must_use]
pub fn default_avatar_path() -> String {
    asset_path(DEFAULT_AVATAR)
}

#[must_use]
pub fn default_character_animations_path() -> String {
    asset_path(DEFAULT_CHARACTER_ANIMATIONS)
}
