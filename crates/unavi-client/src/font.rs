use bevy::prelude::*;
use bevy_msdf::font::asset::{
    FontBytes,
    FontFace,
};
use unavi_assets::{
    FONT_STACK,
    asset_path,
};

/// Requests the fallback chain over iroh. No face is embedded, so text draws
/// nothing until the primary arrives.
pub fn load_font_stack(mut commands: Commands, assets: Res<AssetServer>) {
    for (order, path) in FONT_STACK.iter().enumerate() {
        commands.spawn((
            Name::new(format!("font {path}")),
            FontFace::new(assets.load::<FontBytes>(asset_path(path)), order as u32),
        ));
    }
}
