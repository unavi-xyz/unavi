mod emitter;
mod protocol;
mod receptor;

wired_prelude::generate!();

struct World;

impl exports::unavi::beacon_protocol::api::Guest for World {
    type BeaconEmitter = emitter::BeaconEmitter;
    type BeaconReceptor = receptor::BeaconReceptor;
}

export!(World);
