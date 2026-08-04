use bevy::{
    prelude::*,
    tasks::BoxedFuture,
};
use bevy_hsd::load::{
    LoadHsd,
    OnLoadCtx,
};
use unavi_space::Space;
use unavi_util::async_commands::AsyncCommands;

pub fn join_home(asset_server: Res<AssetServer>, mut commands: Commands) {
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
