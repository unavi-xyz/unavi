use bevy::prelude::*;
use bevy_iroh::IrohEndpoint;

pub const fn register_router_fns(_trigger: On<Add, IrohEndpoint>, _commands: Commands) {
    // commands
    //     .entity(trigger.entity)
    //     .insert(RouterBuilderFns(vec![Box::new(|b| {
    //         let b = b.accept(iroh_gossip::ALPN, gossip.clone());
    //         b
    //     })]));
}

// TODO refactor `wds` crate to not need `router` field in builder
//      + use `bevy_iroh` in `bevy_wds` to construct data stores
