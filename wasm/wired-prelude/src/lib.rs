pub use wired_math;

/// Helper around [`wit_bindgen::generate!`], using better, manually-defined types rather than relying purely on codegen.
#[macro_export]
macro_rules! generate {
    () => {
        ::wit_bindgen::generate!({
            generate_all,
            with: {
                "wired:math/types": ::wired_prelude::wired_math::types,
            },
        });
    };
}

/// Calls [`wired_prelude::generate!`], then adds the needed code for exporting a script component.
/// Pass in a script type, then implement `GuestScript` for the type.
///
/// ## Example
///
/// ```
/// struct Script;
///
/// wired_prelude::generate_script!(Script);
///
/// impl GuestScript for Script {
///     fn new() -> Self {
///         Self
///     }
///     fn tick(&self) {}
///     fn render(&self) {}
///     fn drop(&self) {}
/// }
/// ```
#[macro_export]
macro_rules! generate_script {
    ($script:ident) => {
        ::wired_prelude::generate!();
        struct World;
        impl exports::wired::script::guest_api::Guest for World {
            type Script = $script;
        }
        export!(World);
        use exports::wired::script::guest_api::GuestScript;
    };
}
