use crate::{
    exports::unavi::beacon_protocol::api::GuestBeaconEmitter,
    protocol::CH_BEACON_ID,
    wired::{
        event::{
            api::emit,
            types::{EventFilter, EventScope, SpatialScope},
        },
        scene::types::Node,
    },
};

pub struct BeaconEmitter {
    id: Vec<u8>,
    emitter: Node,
    radius: f32,
}

impl GuestBeaconEmitter for BeaconEmitter {
    fn new(id: Vec<u8>, emitter: Node, radius: f32) -> Self {
        Self {
            id,
            emitter,
            radius,
        }
    }

    fn emit(&self) {
        emit(
            CH_BEACON_ID,
            &self.id,
            EventFilter {
                documents: None,
                scope: EventScope::Spatial(SpatialScope {
                    node: self.emitter.clone(),
                    radius: self.radius,
                }),
            },
        );
    }
}
