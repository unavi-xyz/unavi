mod app;
mod component;
pub mod param;
mod system;

#[allow(clippy::collection_is_never_read)]
mod bindings {
    wit_bindgen::generate!({
        world: "guest",
        path: "../../protocol/wit/wired-ecs",
        additional_derives: [
            bincode::Decode,
            bincode::Encode,
            crate::Component,
        ],
    });
}

pub mod prelude {
    pub use super::{
        App, Component,
        bincode::{Decode, Encode},
        host_api::SystemOrder,
        param::{
            Commands, Entity, Local, Query, Res, ResMut,
            constraint::{With, Without},
        },
        types::{ParamData, Schedule, SystemId},
    };
}

/// Wrapper around [`wit_bindgen::generate`!], with useful defaults.
#[macro_export]
macro_rules! generate_bindgen {
    ( $( $extra:tt )* ) => {
        ::wit_bindgen::generate!({
            generate_all,
            additional_derives: [
                ::wired_ecs::bincode::Decode,
                ::wired_ecs::bincode::Encode,
                ::wired_ecs::Component,
            ],
            with: {
                "wired:ecs/types": ::wired_ecs::types,
            },
            $( $extra )*
        });
    };
}

pub use app::*;
pub use bincode;
pub use bindings::wired::ecs::*;
pub use component::*;
pub use wired_ecs_derive::*;
