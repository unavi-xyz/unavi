//! Faces arriving through Bevy's asset pipeline, so a font can come from
//! wherever an asset source reaches rather than from the binary.

use std::sync::Arc;

use bevy::{
    asset::{
        AssetLoader,
        LoadContext,
        LoadState,
        io::Reader,
    },
    prelude::*,
};

use crate::font::RegisterFont;

/// A font file, held as the bytes a face parses from.
#[derive(Asset, TypePath, Debug)]
pub struct FontBytes(pub Arc<[u8]>);

#[derive(Debug, thiserror::Error)]
pub enum FontBytesError {
    #[error("read font: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Default, TypePath)]
pub struct FontBytesLoader;

impl AssetLoader for FontBytesLoader {
    type Asset = FontBytes;
    type Error = FontBytesError;
    type Settings = ();

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<FontBytes, FontBytesError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(FontBytes(Arc::from(bytes)))
    }

    fn extensions(&self) -> &[&str] {
        &["ttf", "otf", "ttc", "otc"]
    }
}

/// A face requested for the [`crate::font::DefaultFontStack`], joining it once
/// its bytes arrive.
///
/// `order` is the position in the fallback chain, low first. A face waits for
/// every lower order to register before it does, so the chain does not depend
/// on which fetch finishes first — the primary must stay primary even when a
/// fallback downloads faster.
///
/// The entity is a request and is despawned once it resolves, so spawn one per
/// face rather than adding this to an entity that holds anything else.
#[derive(Component, Debug)]
pub struct FontFace {
    pub handle: Handle<FontBytes>,
    pub order:  u32,
}

impl FontFace {
    #[must_use]
    pub const fn new(handle: Handle<FontBytes>, order: u32) -> Self {
        Self { handle, order }
    }
}

pub(crate) fn register_loaded_faces(
    mut commands: Commands,
    faces: Query<(Entity, &FontFace)>,
    bytes: Res<Assets<FontBytes>>,
    assets: Res<AssetServer>,
) {
    let mut waiting = faces.iter().collect::<Vec<_>>();
    waiting.sort_by_key(|(_, face)| face.order);

    for (entity, face) in waiting {
        if let Some(bytes) = bytes.get(&face.handle) {
            commands.trigger(RegisterFont(Arc::clone(&bytes.0)));
            commands.entity(entity).despawn();
            continue;
        }
        if let LoadState::Failed(err) = assets.load_state(&face.handle) {
            error!(
                order = face.order,
                path = ?assets.get_path(&face.handle),
                "font failed to load, leaving its scripts as tofu: {err}"
            );
            commands.entity(entity).despawn();
            continue;
        }
        // A later face must not overtake this one in the chain.
        return;
    }
}

#[cfg(test)]
mod tests {
    use bevy::asset::AssetPlugin;

    use super::*;
    use crate::font::{
        DefaultFontStack,
        on_register_font,
    };

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<FontBytes>()
            .init_asset::<Image>()
            .init_resource::<DefaultFontStack>()
            .add_observer(on_register_font)
            .add_systems(Update, register_loaded_faces);
        app
    }

    fn insert(app: &mut App, bytes: &'static [u8]) -> Handle<FontBytes> {
        app.world_mut()
            .resource_mut::<Assets<FontBytes>>()
            .add(FontBytes(Arc::from(bytes)))
    }

    #[test]
    fn a_loaded_face_joins_the_stack() {
        let mut app = app();
        let handle = insert(&mut app, notosans::REGULAR_TTF);
        app.world_mut().spawn(FontFace::new(handle, 0));

        app.update();
        app.update();

        let stack = app.world().resource::<DefaultFontStack>();
        assert_eq!(stack.0.len(), 1);
        assert!(stack.0[0].state().atlas.can_render('a'));
    }

    /// A fallback that downloads first must not become the primary.
    #[test]
    fn a_face_waits_for_every_lower_order() {
        let mut app = app();
        let fallback = insert(&mut app, notosans::REGULAR_TTF);
        app.world_mut().spawn(FontFace::new(fallback, 1));
        app.world_mut()
            .spawn(FontFace::new(Handle::<FontBytes>::default(), 0));

        app.update();
        app.update();

        assert!(
            app.world().resource::<DefaultFontStack>().0.is_empty(),
            "the chain stays empty while the primary is still loading"
        );
    }
}
