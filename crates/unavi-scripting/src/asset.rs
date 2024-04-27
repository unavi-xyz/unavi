use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::BoxedFuture,
};

#[derive(Asset, Debug, TypePath)]
pub struct Wasm(pub Vec<u8>);

#[derive(Default)]
pub struct WasmLoader;

impl AssetLoader for WasmLoader {
    type Asset = Wasm;
    type Settings = ();
    type Error = std::io::Error;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
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
