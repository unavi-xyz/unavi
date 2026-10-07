//! Shared bindings and sugar for every `hsd/` guest.
//!
//! Merges the former `wired-prelude`, `wired-math`, `wired-scene` and
//! `unavi-script-util` crates into one, so a guest depends on a single
//! `wired-guest` plus `wit-bindgen` itself.
//!
//! A guest calls [`generate_script!`] once, which:
//! - runs `wit_bindgen::generate!` for its own world, mapping `wired:core/math`
//!   onto [`math`] so `vec2`/`vec3` are `glam`'s own types;
//! - defines a local `ScriptBehavior` trait (`init`, `update(tick)`,
//!   `fixed_update(tick)`) and wires it to `wired:script/lifecycle`;
//! - adds `Document::local()`/`Document::shared()` batchers, `find_one`, a
//!   `camera_pose` helper, and `Default`/builders for `Material`, `RigidBody`
//!   and `Text`.
//!
//! These additions live in the calling crate's own root module (the macro
//! expands there), because the resource and record types they extend are
//! generated fresh by each crate's own `wit_bindgen::generate!` call and
//! cannot be named from here ahead of time. [`math`] is the exception: it is
//! plain data shared by every guest through the `with:` mapping, so it is a
//! real, inherent part of this crate's public API.

pub mod beacon;
pub mod color;
pub mod math;
pub mod xform;

pub use wired_state_macro::state;

/// [`wit_bindgen::generate!`] for a guest's own world, plus the ergonomics.
///
/// Documented on the crate itself. Call this directly only for a `library`
/// world (no lifecycle export); a `script` world should use
/// [`generate_script!`] instead.
#[macro_export]
macro_rules! generate {
    () => {
        ::wit_bindgen::generate!({
            generate_all,
            with: {
                "wired:core/math": ::wired_guest::math,
            },
        });

        /// Batches edits to a [`wired::scene::document::Document`], flushed
        /// as one `apply` call. Built by [`wired::scene::document::Document::local`]
        /// or [`wired::scene::document::Document::shared`].
        pub struct Batch<'a> {
            doc:   &'a wired::scene::document::Document,
            layer: wired::scene::document::Layer,
            edits: ::std::vec::Vec<wired::scene::document::Edit>,
        }

        impl<'a> Batch<'a> {
            /// Queues setting `property` on `prim`.
            #[must_use]
            pub fn set(
                mut self,
                prim: (u64, u64),
                property: wired::scene::properties::Property,
            ) -> Self {
                self.edits
                    .push(wired::scene::document::Edit::Set((prim, property)));
                self
            }

            /// Queues dropping this layer's value for `key` on `prim`.
            #[must_use]
            pub fn clear(
                mut self,
                prim: (u64, u64),
                key: wired::scene::properties::PropertyKey,
            ) -> Self {
                self.edits
                    .push(wired::scene::document::Edit::Clear((prim, key)));
                self
            }

            /// Queues taking `prim` and its subtree out of the scene.
            #[must_use]
            pub fn remove(mut self, prim: (u64, u64)) -> Self {
                self.edits.push(wired::scene::document::Edit::Remove(prim));
                self
            }

            /// Applies every queued edit together, or none of them. A batch
            /// with no edits does nothing and never calls the host.
            pub fn flush(self) -> ::core::result::Result<(), wired::core::error::Error> {
                if self.edits.is_empty() {
                    return ::core::result::Result::Ok(());
                }
                self.doc.apply(self.layer, &self.edits)
            }
        }

        impl wired::scene::document::Document {
            /// Starts a batch written to this peer only.
            #[must_use]
            pub fn local(&self) -> Batch<'_> {
                Batch {
                    doc:   self,
                    layer: wired::scene::document::Layer::Local,
                    edits: ::std::vec::Vec::new(),
                }
            }

            /// Starts a batch replicated to every present peer. Lands only
            /// on the peer whose user authors this document; see
            /// `wired:scene/document.layer`.
            #[must_use]
            pub fn shared(&self) -> Batch<'_> {
                Batch {
                    doc:   self,
                    layer: wired::scene::document::Layer::Shared,
                    edits: ::std::vec::Vec::new(),
                }
            }
        }

        /// The first prim named `name` in `doc`, or `not-found` when there
        /// is none.
        pub fn find_one(
            doc: &wired::scene::document::Document,
            name: &str,
        ) -> ::core::result::Result<(u64, u64), wired::core::error::Error> {
            doc.find_by_name(name)
                .into_iter()
                .next()
                .ok_or(wired::core::error::Error::NotFound)
        }

        /// The local user's camera pose this frame, or `None` until the
        /// avatar has loaded.
        #[must_use]
        pub fn camera_pose() -> ::core::option::Option<::wired_guest::math::Transform> {
            wired::agent::local::camera_transform().ok()
        }

        impl ::core::default::Default for wired::scene::properties::Material {
            fn default() -> Self {
                Self {
                    base_color:   ::core::option::Option::None,
                    emissive:     ::core::option::Option::None,
                    metallic:     ::core::option::Option::None,
                    roughness:    ::core::option::Option::None,
                    alpha_mode:   ::core::option::Option::None,
                    alpha_cutoff: ::core::option::Option::None,
                    double_sided: ::core::option::Option::None,
                }
            }
        }

        impl wired::scene::properties::Material {
            /// A material with only `base-color` set, every other field
            /// taking the renderer's default.
            #[must_use]
            pub fn solid(color: ::wired_guest::math::Color) -> Self {
                Self {
                    base_color: ::core::option::Option::Some(color),
                    ..::core::default::Default::default()
                }
            }
        }

        impl wired::scene::properties::RigidBody {
            /// `kind`, with every other field taking the simulation's
            /// default.
            #[must_use]
            pub const fn new(kind: wired::scene::properties::RigidBodyKind) -> Self {
                Self {
                    kind,
                    mass: ::core::option::Option::None,
                    friction: ::core::option::Option::None,
                    restitution: ::core::option::Option::None,
                    linear_damping: ::core::option::Option::None,
                    angular_damping: ::core::option::Option::None,
                }
            }

            #[must_use]
            pub const fn dynamic() -> Self {
                Self::new(wired::scene::properties::RigidBodyKind::Dynamic)
            }

            #[must_use]
            pub const fn kinematic() -> Self {
                Self::new(wired::scene::properties::RigidBodyKind::Kinematic)
            }

            #[must_use]
            pub const fn static_body() -> Self {
                Self::new(wired::scene::properties::RigidBodyKind::Static)
            }
        }

        impl wired::scene::properties::Text {
            /// `value`, with every other field taking the renderer's
            /// default.
            #[must_use]
            pub fn new(value: impl ::core::convert::Into<::std::string::String>) -> Self {
                Self {
                    value: value.into(),
                    size: ::core::option::Option::None,
                    align: ::core::option::Option::None,
                    anchor: ::core::option::Option::None,
                    wrap: ::core::option::Option::None,
                    line_height: ::core::option::Option::None,
                    color: ::core::option::Option::None,
                    outline: ::core::option::Option::None,
                    outline_width: ::core::option::Option::None,
                    emissive: ::core::option::Option::None,
                    billboard: ::core::option::Option::None,
                }
            }
        }
    };
}

/// [`generate!`], plus a local `ScriptBehavior` trait and the glue that
/// drives `$script: ScriptBehavior` from `wired:script/lifecycle`.
///
/// ```ignore
/// wired_guest::generate_script!(Script);
///
/// struct Script;
///
/// impl ScriptBehavior for Script {
///     fn init() -> anyhow::Result<Self> {
///         Ok(Self)
///     }
/// }
/// ```
#[macro_export]
macro_rules! generate_script {
    ($script:ident) => {
        $crate::generate!();

        /// A script's state. `init` runs once; `update` and `fixed_update`
        /// each carry the `tick` that triggered them.
        pub trait ScriptBehavior: ::core::marker::Sized {
            /// Called once, before any other export. An error retires the
            /// script, with the message logged by the host.
            fn init() -> ::anyhow::Result<Self>;

            /// Called at the host's fixed rate. Input, messages and
            /// simulation belong here.
            #[expect(unused_variables)]
            fn fixed_update(
                &mut self,
                tick: exports::wired::script::lifecycle::Tick,
            ) -> ::anyhow::Result<()> {
                ::core::result::Result::Ok(())
            }

            /// Called once per rendered frame. Drawing and animation belong
            /// here.
            #[expect(unused_variables)]
            fn update(
                &mut self,
                tick: exports::wired::script::lifecycle::Tick,
            ) -> ::anyhow::Result<()> {
                ::core::result::Result::Ok(())
            }
        }

        ::std::thread_local! {
            static __SCRIPT: ::std::cell::RefCell<::std::option::Option<$script>> =
                ::std::cell::RefCell::new(::core::option::Option::None);
        }

        struct World;

        impl exports::wired::script::lifecycle::Guest for World {
            fn init() -> ::core::result::Result<(), ::std::string::String> {
                match <$script as ScriptBehavior>::init() {
                    ::core::result::Result::Ok(state) => {
                        __SCRIPT.with(|s| *s.borrow_mut() = ::core::option::Option::Some(state));
                        ::core::result::Result::Ok(())
                    }
                    ::core::result::Result::Err(err) => {
                        ::core::result::Result::Err(::std::string::ToString::to_string(&err))
                    }
                }
            }

            fn fixed_update(tick: exports::wired::script::lifecycle::Tick) {
                __SCRIPT.with(|s| {
                    if let ::core::option::Option::Some(state) = s.borrow_mut().as_mut()
                        && let ::core::result::Result::Err(err) = state.fixed_update(tick)
                    {
                        ::std::eprintln!("script fixed update: {err:?}");
                    }
                });
            }

            fn update(tick: exports::wired::script::lifecycle::Tick) {
                __SCRIPT.with(|s| {
                    if let ::core::option::Option::Some(state) = s.borrow_mut().as_mut()
                        && let ::core::result::Result::Err(err) = state.update(tick)
                    {
                        ::std::eprintln!("script update: {err:?}");
                    }
                });
            }
        }

        export!(World);
    };
}
