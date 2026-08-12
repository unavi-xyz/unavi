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

/// A content-addressed file the client fetches over iroh.
///
/// An iroh blob hash is the blake3 hash of the content, so `b3sum <file>`
/// verifies an entry against what a server reports in its `files.json`. The
/// store verifies downloads against the hash, so the manifest needs no size.
#[derive(Debug, Clone, Copy)]
pub struct AssetSpec {
    /// The path the hosting server uses in its files directory.
    pub rel_path: &'static str,
    pub hash:     &'static str,
}

/// The assets the client pulls over iroh. Every entry must be hosted by a
/// reachable unavi-server (or other provider).
pub const MANIFEST: &[AssetSpec] = &[
    AssetSpec {
        rel_path: DEFAULT_AVATAR,
        hash:     "a2f1a48db6cdf369ab510f6a6fb869d107897231b70c4920ad0357e4930c6281",
    },
    AssetSpec {
        rel_path: DEFAULT_CHARACTER_ANIMATIONS,
        hash:     "9fbda809b00ab14e58356721e0c0a92fe88b9234c486a43b9417c4f27555c0c6",
    },
    AssetSpec {
        rel_path: DEFAULT_FONT,
        hash:     "3a21ac778bcc91b57dc32576c6baffbcb493b78b4b6ad46b05c3d33bb5da7315",
    },
    AssetSpec {
        rel_path: CJK_FONT,
        hash:     "1580ba0d54c84191041a55ec8d442d5a7d3668e5af8c9fee5456c776c30ff16a",
    },
];

/// The manifest entry for a relative path, if any.
#[must_use]
pub fn manifest_entry(rel_path: &str) -> Option<&'static AssetSpec> {
    MANIFEST.iter().find(|asset| asset.rel_path == rel_path)
}

/// The path Bevy loads a manifest asset by, on every platform.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_named_asset_is_hosted() {
        for rel_path in FONT_STACK
            .iter()
            .copied()
            .chain([DEFAULT_AVATAR, DEFAULT_CHARACTER_ANIMATIONS])
        {
            assert!(
                manifest_entry(rel_path).is_some(),
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
