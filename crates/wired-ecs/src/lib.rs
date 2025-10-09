mod app;
mod component;
pub mod param;
mod system;

mod bindings {
    wit_bindgen::generate!({
        world: "guest",
        path: "../../protocol/wit/wired-ecs",
    });
}

pub mod prelude {
    pub use super::{
        App, Component,
        bindings::wired::ecs::host_api::SystemOrder,
        param::{
            Commands, Entity, Local, Query, Res, ResMut,
            constraint::{With, Without},
        },
        types::{ParamData, Schedule, SystemId},
    };
}

pub use app::*;
pub use bincode;
pub use bindings::wired::ecs::*;
pub use component::*;
pub use wired_ecs_derive::*;
