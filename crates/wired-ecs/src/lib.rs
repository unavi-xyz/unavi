mod app;
mod component;
pub mod param;
mod system;

#[allow(clippy::unsafe_derive_deserialize)]
mod bindings {
    wit_bindgen::generate!({
        world: "guest",
        path: "../../protocol/wit/wired-ecs",
        additional_derives: [
            serde::Serialize,
            serde::Deserialize,
            crate::Component,
        ],
    });
}

/// Wrapper around [`wit_bindgen::generate`!], with useful defaults.
#[macro_export]
macro_rules! generate_bindgen {
    ( $( $extra:tt )* ) => {
        #[allow(clippy::unsafe_derive_deserialize)]
        mod bindings {
            ::wit_bindgen::generate!({
                generate_all,
                additional_derives: [
                    ::serde::Serialize,
                    ::serde::Deserialize,
                    ::wired_ecs::Component,
                ],
                with: {
                    "wired:ecs/types": ::wired_ecs::types,
                },
                $( $extra )*
            });
        }
        use bindings::{export, exports};
    };
}

pub use app::*;
pub use bindings::wired::ecs::*;
pub use component::*;
pub use wired_ecs_derive::*;

pub mod prelude {
    pub use super::{
        App, Component, component,
        host_api::SystemOrder,
        param::{
            Commands, Entity, Local, Query, Res, ResMut,
            constraint::{With, Without},
        },
        types::{ParamData, Schedule, SystemId},
    };
}
