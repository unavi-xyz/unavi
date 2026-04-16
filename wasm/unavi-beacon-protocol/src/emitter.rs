use crate::{
    exports::unavi::beacon_protocol::api::GuestBeaconEmitter,
    protocol::CH_BEACON_ID,
    wired::event::{
        api::emit,
        types::{EventFilter, EventScope},
    },
};

pub struct BeaconEmitter {
    id: Vec<u8>,
}

impl GuestBeaconEmitter for BeaconEmitter {
    fn new(id: Vec<u8>) -> Self {
        Self { id }
    }

    fn emit(&self) {
        emit(
            CH_BEACON_ID,
            &self.id,
            EventFilter {
                node: None,
                scope: EventScope::Global,
                documents: None,
            },
        );
    }
}
