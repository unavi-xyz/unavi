pub use wired_kv;
pub use wired_math;
pub use wired_scene;

pub mod prelude {
    pub use wired_math::types::*;
    pub use wired_scene::types::*;
}

pub trait ScriptBehavior: Sized {
    /// Called once to initialize the script.
    fn init() -> Self;
    /// Called on a fixed interval.
    fn tick(&mut self) {}
    /// Called every frame before rendering.
    fn render(&mut self) {}
}

/// Helper around [`wit_bindgen::generate!`], using better, manually-defined
/// types rather than relying purely on codegen.
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
                wired::kv::api::self_kv()
            }
            fn get_kv(doc_id: &[u8]) -> ::core::option::Option<Self> {
                wired::kv::api::get_kv(doc_id)
            }
            fn kv_get(&self, key: &str) -> ::core::option::Option<::std::vec::Vec<u8>> {
                self.get(key)
            }
            fn kv_set(
                &self,
                key: &str,
                value: &[u8],
            ) -> ::core::result::Result<(), ::wired_prelude::wired_kv::KvErrorKind> {
                self.set(key, value).map_err(|e| match e {
                    wired::kv::types::KvError::QuotaExceeded =>
                        ::wired_prelude::wired_kv::KvErrorKind::QuotaExceeded,
                    wired::kv::types::KvError::KeyTooLong =>
                        ::wired_prelude::wired_kv::KvErrorKind::KeyTooLong,
                })
            }
            fn kv_delete(&self, key: &str) {
                self.delete(key);
            }
            fn kv_keys(&self) -> ::std::vec::Vec<::std::string::String> {
                self.keys()
            }
        }
    };
}

/// Calls [`wired_prelude::generate!`], then wires up the script exports to a
/// provided type.
///
/// ## Example
///
/// ```
/// struct Script;
///
/// wired_prelude::generate_script!(Script);
///
/// impl ScriptBehavior for Script {
///     fn init() -> Self {
///         Self
///     }
/// }
/// ```
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
                __SCRIPT.with(|s| *s.borrow_mut() = Some($script::init()));
            }
            fn tick() {
                __SCRIPT.with(|s| {
                    if let Some(state) = s.borrow_mut().as_mut() {
                        state.tick();
                    }
                });
            }
            fn render() {
                __SCRIPT.with(|s| {
                    if let Some(state) = s.borrow_mut().as_mut() {
                        state.render();
                    }
                });
            }
        }
        export!(World);
    };
}
