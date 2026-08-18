pub use wired_kv;
pub use wired_math;
pub use wired_scene;

pub mod prelude {
    pub use wired_math::types::*;
    pub use wired_scene::types::*;
}

pub trait ScriptBehavior: Sized {
    /// Called once to initialize the script.
    fn init() -> ::anyhow::Result<Self>;
    /// Called on a fixed interval.
    fn fixed_update(&mut self) -> ::anyhow::Result<()> {
        Ok(())
    }
    /// Called every frame before rendering.
    fn update(&mut self) -> ::anyhow::Result<()> {
        Ok(())
    }
}

/// [`wit_bindgen::generate!`] with manually-defined types in place of codegen.
#[macro_export]
macro_rules! generate {
    () => {
        ::wit_bindgen::generate!({
            generate_all,
            with: {
                "wired:math/types": ::wired_prelude::wired_math::types,
                "wired:scene/types/color": ::wired_prelude::wired_scene::types::Color,
            },
        });

        impl ::wired_prelude::wired_kv::WiredKv for wired::kv::types::Kv {
            fn self_kv() -> Self {
                wired::kv::api::self_kv().expect("self_kv")
            }
            fn get_kv(doc_id: &[u8]) -> ::core::option::Option<Self> {
                wired::kv::api::get_kv(doc_id).ok().flatten()
            }
            fn kv_get(&self, key: &str) -> ::core::option::Option<::std::vec::Vec<u8>> {
                self.get(key)
            }
            fn kv_set(
                &self,
                key: &str,
                value: &[u8],
            ) -> ::core::result::Result<(), ::wired_prelude::wired_kv::TypedKvError> {
                self.set(key, value).map_err(|e| match e {
                    wired::error::types::Error::QuotaFlow =>
                        ::wired_prelude::wired_kv::TypedKvError::QuotaFlow,
                    wired::error::types::Error::QuotaStock =>
                        ::wired_prelude::wired_kv::TypedKvError::QuotaStock,
                    wired::error::types::Error::Permission =>
                        ::wired_prelude::wired_kv::TypedKvError::Permission,
                    wired::error::types::Error::Reach =>
                        ::wired_prelude::wired_kv::TypedKvError::Reach,
                    wired::error::types::Error::Other(detail) =>
                        ::wired_prelude::wired_kv::TypedKvError::Other(detail),
                })
            }
            fn kv_delete(&self, key: &str) {
                let _ = self.delete(key);
            }
            fn kv_keys(&self) -> ::std::vec::Vec<::std::string::String> {
                self.keys()
            }
        }
    };
}

#[macro_export]
macro_rules! generate_script {
    ($script:ident) => {
        ::wired_prelude::generate!();
        use ::wired_prelude::ScriptBehavior;

        ::std::thread_local! {
            static __SCRIPT: ::std::cell::RefCell<::std::option::Option<$script>> =
                ::std::cell::RefCell::new(None);
        }

        struct World;
        impl exports::wired::script::guest_api::Guest for World {
            fn init() {
                match $script::init() {
                    ::core::result::Result::Ok(state) => {
                        __SCRIPT.with(|s| *s.borrow_mut() = ::core::option::Option::Some(state));
                    }
                    ::core::result::Result::Err(err) => ::std::eprintln!("script init: {err:?}"),
                }
            }
            fn fixed_update() {
                __SCRIPT.with(|s| {
                    if let Some(state) = s.borrow_mut().as_mut()
                        && let ::core::result::Result::Err(err) = state.fixed_update()
                    {
                        ::std::eprintln!("script fixed update: {err:?}");
                    }
                });
            }
            fn update() {
                __SCRIPT.with(|s| {
                    if let Some(state) = s.borrow_mut().as_mut()
                        && let ::core::result::Result::Err(err) = state.update()
                    {
                        ::std::eprintln!("script update: {err:?}");
                    }
                });
            }
        }
        export!(World);
    };
}
