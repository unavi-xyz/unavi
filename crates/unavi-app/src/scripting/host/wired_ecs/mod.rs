use bevy::prelude::*;
use crossbeam::channel::Receiver;

mod handler;
mod linker;
mod query;

pub use handler::*;
pub use linker::*;
pub use query::*;

#[derive(Component)]
pub struct WiredEcsReceiver(pub Receiver<WiredEcsCommand>);

pub enum WiredEcsCommand {
    Insert {
        entity: u32,
        instance: u32,
    },
    RegisterComponent {
        id: u32,
    },
    RegisterQuery {
        id: u32,
        components: Vec<u32>,
    },
    Spawn {
        id: u32,
        components: Vec<InstanceResource>,
    },
}
