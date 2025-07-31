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

pub use app::*;
pub use bindings::wired::ecs::*;
pub use bytemuck;
pub use component::*;
pub use wired_ecs_derive::*;
