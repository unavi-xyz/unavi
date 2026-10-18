//! The `.wasm` asset a script runs.

use std::sync::Arc;

use bevy::{
    asset::{
        AssetLoader,
        LoadContext,
        LoadState,
        io::Reader,
    },
    prelude::*,
    reflect::TypePath,
    tasks::ConditionalSendFuture,
};

use crate::{
    Script,
    ScriptStatus,
};

/// A script's component bytes, and their hash, which compiled code is cached
/// by.
#[derive(Asset, Debug, TypePath)]
pub struct Wasm {
    pub bytes: Arc<[u8]>,
    pub hash:  blake3::Hash,
}

impl Wasm {
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            hash:  blake3::hash(&bytes),
            bytes: bytes.into(),
        }
    }
}

/// Retires a script whose `.wasm` failed to load, which would otherwise wait
/// for bytes that never arrive.
pub fn trap_failed_loads(
    scripts: Query<(&Script, &ScriptStatus, NameOrEntity)>,
    asset_server: Res<AssetServer>,
) {
    for (script, status, name) in &scripts {
        if status.is_settled() {
            continue;
        }
        if matches!(
            asset_server.get_load_state(&script.0),
            Some(LoadState::Failed(_))
        ) && status.trap()
        {
            error!(name = %name, "Failed to load the script; it will not run");
        }
    }
}

#[derive(Default, TypePath)]
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
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            Ok(Wasm::new(bytes))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["wasm"]
    }
}
