#![allow(dead_code)]

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use bevy::{log::LogPlugin, prelude::*};
use bevy_hsd::HsdPlugin;
use bevy_wds::{LocalActor, LocalBlobs, WdsPlugin};
use iroh::{endpoint::presets::N0, protocol::Router};
use tracing_subscriber::Layer;
use unavi_script::{ScriptPlugin, load::local::LoadLocalScript, permissions::ApiPermissions};
use unavi_util::async_task::spawn_async_task;
use wds::{Blobs, DataStore, Identity, actor::Actor};
use xdid::methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair};

use crate::setup::logs::LOGS;

pub mod logs;

const TICK: Duration = Duration::from_millis(100);

pub fn setup_test_app(package: &'static str, perms: ApiPermissions) -> App {
    let (actor, blobs) = create_test_wds();

    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin {
            file_path: "../unavi-client/assets".to_string(),
            ..Default::default()
        },
        LogPlugin {
            custom_layer: |_| Some(LOGS.clone().boxed()),
            ..Default::default()
        },
        WdsPlugin,
        HsdPlugin,
        ScriptPlugin,
    ))
    .init_asset::<StandardMaterial>()
    .init_asset::<Mesh>()
    .insert_resource(Time::<Virtual>::from_max_delta(TICK))
    .insert_resource(Time::<Fixed>::from_duration(TICK));

    app.world_mut()
        .spawn((LocalActor(actor), LocalBlobs(blobs)));

    app.world_mut()
        .spawn(perms)
        .trigger(|entity| LoadLocalScript {
            entity,
            path: format!("wasm/test/{package}.wasm"),
        });

    app
}

/// Poll `app` until `condition()` returns true or `timeout` elapses.
///
/// Returns `true` if the condition was met, `false` on timeout.
pub fn wait_until(app: &mut App, condition: impl Fn() -> bool, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        tick_app(app);
        if condition() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
    }
}

pub fn tick_app(app: &mut App) {
    app.update();
    // Sleep to allow async work to run and for virtual time to advance by TICK.
    std::thread::sleep(Duration::from_millis(300));
}

#[must_use]
pub fn create_test_wds() -> (Actor, Blobs) {
    let (tx, rx) = async_channel::bounded(1);

    spawn_async_task(async move {
        let endpoint = iroh::Endpoint::builder(N0)
            .bind()
            .await
            .expect("iroh endpoint");

        let (store, f) = DataStore::builder(endpoint.clone())
            .build()
            .await
            .expect("data store");

        let rb = Router::builder(endpoint);
        let rb = f(rb);
        let _router = rb.spawn();

        let blobs = store.blobs().blobs().clone();

        let signing_key = P256KeyPair::generate();
        let did = signing_key.public().to_did();
        let identity = Arc::new(Identity::new(did, signing_key));
        let actor = store.local_actor(identity);

        tx.send((actor, blobs)).await.expect("send");
    });

    rx.recv_blocking().expect("wds setup")
}
