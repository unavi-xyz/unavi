mod api;

#[cfg(not(target_family = "wasm"))]
pub mod native;

pub struct StoreState {
    #[cfg(not(target_family = "wasm"))]
    pub native: native::NativeStoreState,
}
