use std::str::FromStr;

use bevy::{
    prelude::*,
    tasks::BoxedFuture,
};
use bevy_hsd::load::{
    LoadHsd,
    OnLoadCtx,
};
use bevy_wds::doc::DocSet;
use iroh_docs::NamespaceId;
use unavi_policy::space::Space;
use unavi_util::async_commands::AsyncCommands;

/// A namespace to enter instead of the local home, from `--join`.
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
        record_home(ctx.namespace).await;
        Ok(())
    })
}

/// Key under a DID's root doc naming the space it comes home to.
const HOME_KEY: &str = "home";

/// Version prefix on the entry, so a reader cannot misread its bytes as a
/// namespace.
const HOME_VERSION: u32 = 0;

/// Writes down which space is home, so a shell can offer to travel back to it;
/// without this entry a script has no way to learn it.
async fn record_home(ns: NamespaceId) {
    let Some(root) = bevy_wds::root_doc() else {
        return;
    };
    let mut value = match postcard::to_stdvec(&HOME_VERSION) {
        Ok(value) => value,
        Err(err) => {
            warn!(?err, "could not encode the home space ref");
            return;
        }
    };
    value.extend_from_slice(ns.as_bytes());

    let (tx, rx) = async_channel::bounded(1);
    if AsyncCommands::default()
        .trigger(DocSet {
            ns: root,
            key: HOME_KEY.to_string(),
            value: value.into(),
            tx,
        })
        .send()
        .await
        .is_err()
    {
        return;
    }
    if rx.recv().await == Ok(true) {
        info!(%ns, "Recorded home space");
    } else {
        warn!("could not record the home space");
    }
}
