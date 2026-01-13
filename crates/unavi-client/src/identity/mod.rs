#[cfg(not(target_family = "wasm"))]
mod key_pair;

#[cfg(not(target_family = "wasm"))]
pub use key_pair::get_or_create_key;
