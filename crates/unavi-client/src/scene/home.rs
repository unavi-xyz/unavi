use std::str::FromStr;

use bevy::{
    prelude::*,
    tasks::BoxedFuture,
};
use bevy_hsd::load::{
    LoadHsd,
    OnLoadCtx,
};
use iroh_docs::NamespaceId;
use unavi_space::Space;
use unavi_util::async_commands::AsyncCommands;

/// A namespace to enter instead of the local home, from `--join`.
///
/// Reaching another peer's space otherwise requires walking a portal, which no
/// automated run can do — so a multiplayer bug can only be reproduced by hand.
#[derive(Resource, Default)]
pub struct JoinSpace(pub Option<String>);

pub fn join_startup_space(
    join: Res<JoinSpace>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let Some(raw) = join.0.as_deref() else {
        join_home(&asset_server, &mut commands);
        return;
    };

    match NamespaceId::from_str(raw) {
        Ok(ns) => {
            info!(%ns, "Joining space");
            commands.spawn(Space(ns));
        }
        Err(err) => {
            error!(?err, raw, "Invalid --join namespace, falling back to home");
            join_home(&asset_server, &mut commands);
        }
    }
}

pub fn join_home(asset_server: &AssetServer, commands: &mut Commands) {
    let handle = asset_server.load("hsd/unavi_default_home.hsdz");
    commands.spawn(LoadHsd {
        handle,
        on_load: Some(Box::new(on_load_spawn_space)),
    });
}

#[must_use]
pub fn on_load_spawn_space(ctx: OnLoadCtx) -> BoxedFuture<'static, anyhow::Result<()>> {
    info!(id = %ctx.namespace, "Joining home");
    Box::pin(async move {
        AsyncCommands::default()
            .push(move |world: &mut World| {
                world.entity_mut(ctx.entity).insert(Space(ctx.namespace));
            })
            .send()
            .await?;
        Ok(())
    })
}
