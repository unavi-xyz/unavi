use bevy::prelude::*;

pub mod hsd;
pub mod init;
pub mod log;
pub mod state;

pub mod bindings {
    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-script",
        imports: {
            default: async,
        },
        exports: {
            default: async,
        }
    });
}

pub type LoadResult = wasmtime::Result<(Entity, bindings::Guest)>;

#[derive(Component)]
pub struct LoadingScript;

#[derive(Component)]
#[require(Executing)]
pub struct LoadedScript(pub std::sync::Arc<bindings::Guest>);

#[derive(Component, Default, Deref, DerefMut)]
pub struct Executing(bool);
