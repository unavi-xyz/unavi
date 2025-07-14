use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
    reflect::TypePath,
};

#[derive(Asset, Debug, Deref, DerefMut, TypePath)]
pub struct Wasm(pub Vec<u8>);

#[derive(Default)]
pub struct WasmLoader;

impl AssetLoader for WasmLoader {
    type Asset = Wasm;
    type Settings = ();
    type Error = std::io::Error;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl bevy::tasks::ConditionalSendFuture<Output = std::result::Result<Self::Asset, Self::Error>>
    {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            Ok(Wasm(bytes))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["wasm"]
    }
}
