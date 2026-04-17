use crate::{
    exports::unavi::beacon_protocol::api::GuestBeaconReceptor,
    protocol::CH_BEACON_ID,
    wired::{
        event::{
            api::listen,
            types::{EventFilter, EventReceptor, EventScope},
        },
        scene::types::Node,
    },
};

pub struct BeaconReceptor {
    receptor: EventReceptor,
}

impl GuestBeaconReceptor for BeaconReceptor {
    fn new(receptor: Node, radius: f32) -> Self {
        let receptor = listen(
            &[CH_BEACON_ID.to_string()],
            EventFilter {
                node: Some(receptor),
                scope: EventScope::Spatial(radius),
                documents: None,
            },
        );
        Self { receptor }
    }

    fn poll(&self) -> Option<Vec<u8>> {
        self.receptor.poll().map(|e| e.payload)
    }
}
