use crate::{
    exports::unavi::beacon_protocol::api::GuestBeaconReceptor,
    protocol::CH_BEACON_ID,
    wired::{
        event::{
            api::listen,
            types::{EventFilter, EventReceptor, EventScope, SpatialScope},
        },
        scene::types::Node,
    },
};

pub struct BeaconReceptor {
    receptor: EventReceptor,
}

impl GuestBeaconReceptor for BeaconReceptor {
    fn new(node: Node, radius: f32) -> Self {
        let receptor = listen(
            &[CH_BEACON_ID.to_string()],
            EventFilter {
                documents: None,
                scope: EventScope::Spatial(SpatialScope { node, radius }),
            },
        );
        Self { receptor }
    }

    fn poll(&self) -> Option<Vec<u8>> {
        self.receptor.poll().map(|e| e.payload)
    }
}
