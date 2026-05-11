use std::sync::Arc;

use bevy::{prelude::*, tasks::BoxedFuture};
use bevy_hsd::load::{LoadHsd, OnLoadCtx};
use bevy_wds::record::write::SchemaDef;
use unavi_space::Space;
use unavi_util::async_commands::AsyncCommands;
use wired_schemas::{SCHEMA_HOME, SCHEMA_SPACE};

pub fn join_home(asset_server: Res<AssetServer>, mut commands: Commands) {
    let handle = asset_server.load("hsd/unavi_default_home.hsd");
    commands.spawn(LoadHsd {
        handle,
        extra_schemas: Some(vec![
            SchemaDef {
                container: "home".into(),
                schema: (&*SCHEMA_HOME).into(),
                f: Arc::new(|_| Ok(())),
            },
            SchemaDef {
                container: "space".into(),
                schema: (&*SCHEMA_SPACE).into(),
                f: Arc::new(|doc| {
                    let map = doc.get_map("space");
                    map.insert("name", "My Home".to_string())?;
                    Ok(())
                }),
            },
        ]),
        on_load: Some(Box::new(on_load_spawn_space)),
    });
}

#[must_use]
pub fn on_load_spawn_space(ctx: OnLoadCtx) -> BoxedFuture<'static, anyhow::Result<()>> {
    info!(id = %ctx.record_id, "Joining home");
    Box::pin(async move {
        AsyncCommands::default()
            .push(move |world: &mut World| {
                world.entity_mut(ctx.entity).insert(Space(ctx.record_id));
            })
            .send()
            .await?;
        Ok(())
    })
}
