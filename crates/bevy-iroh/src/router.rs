use bevy::prelude::*;
use iroh::protocol::RouterBuilder;

use crate::{BuildRouter, IrohEndpoint, IrohRouter, RouterBuilderFn, RouterBuilderFns};

pub fn on_build_router(
    trigger: On<BuildRouter>,
    mut commands: Commands,
    endpoints: Query<&IrohEndpoint>,
    mut builders: Query<&mut RouterBuilderFns>,
    mut fs: Query<&mut RouterBuilderFn>,
) {
    let entity = trigger.event().event_target();

    let Ok(endpoint) = endpoints.get(entity).map(|v| v.0.clone()) else {
        warn!(%entity, "cannot build router, endpoint not found");
        return;
    };
    let Ok(fns) = builders.get_mut(entity) else {
        warn!(%entity, "cannot build router, protocols not found");
        return;
    };

    let mut builder = RouterBuilder::new(endpoint);

    for fn_ent in &fns.0 {
        let mut f = fs.get_mut(*fn_ent).expect("router builder");

        commands.entity(*fn_ent).despawn();
        let Some(f) = f.0.take() else {
            continue;
        };

        builder = f(builder);
    }

    let router = builder.spawn();

    commands.entity(entity).insert(IrohRouter(router));
}
