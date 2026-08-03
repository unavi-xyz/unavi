#![allow(dead_code)]

use std::{
    sync::Arc,
    time::Duration,
};

use avian3d::PhysicsPlugins;
use bevy::{
    asset::AssetPlugin,
    prelude::*,
    transform::TransformPlugin,
};
use bevy_wds::{
    LocalBlobs,
    WdsPlugin,
};
use iroh_blobs::{
    api::blobs::Blobs,
    store::mem::MemStore,
};
use loro::LoroDoc;
use rstest::fixture;
use unavi_util::async_task::spawn_async_task;

pub struct TestContext {
    pub app: App,
    pub doc: Arc<LoroDoc>,
    blobs:   Option<Blobs>,
}

impl Default for TestContext {
    fn default() -> Self {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            TransformPlugin,
            bevy_hsd::HsdPlugin,
        ))
        .init_asset::<Image>()
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>();

        let mut ctx = Self {
            app,
            doc: Arc::default(),
            blobs: None,
        };

        ctx.spawn_hsd();

        ctx
    }
}

impl TestContext {
    /// Same as `default()` but with avian's [`PhysicsPlugins`] enabled.
    /// Use this for any test that exercises colliders or rigid bodies —
    /// avian's `On<Add, Collider>` observer reads `Position` / `Rotation`
    /// and will panic on the placeholder MAX values if our `apply_collider`
    /// ever inserts a `Collider` without seeding them.
    pub fn with_physics() -> Self {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            TransformPlugin,
            bevy::scene::ScenePlugin,
            PhysicsPlugins::default(),
            bevy_hsd::HsdPlugin,
        ))
        .init_asset::<Image>()
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_secs_f32(1.0 / 60.0),
        ));
        app.finish();
        app.cleanup();

        let mut ctx = Self {
            app,
            doc: Arc::default(),
            blobs: None,
        };

        ctx.spawn_hsd();

        ctx
    }

    pub fn with_wds() -> Self {
        let blobs = setup_blobs();

        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            TransformPlugin,
            WdsPlugin,
            bevy_hsd::HsdPlugin,
        ))
        .init_asset::<Image>()
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(10)));

        app.world_mut().spawn(LocalBlobs(blobs.clone()));

        let mut ctx = Self {
            app,
            doc: Arc::default(),
            blobs: Some(blobs),
        };

        ctx.spawn_hsd();

        ctx
    }

    pub fn spawn_hsd(&mut self) {
        self.app
            .world_mut()
            .spawn(bevy_hsd::Hsd(Arc::clone(&self.doc)));
    }

    /// Upload `bytes` to the local blob store and return its hash.
    /// Panics if the context was not created with `with_wds`.
    pub fn upload_blob(&self, bytes: Vec<u8>) -> blake3::Hash {
        let blobs = self.blobs.clone().expect("wds not enabled");
        let hash = blake3::hash(&bytes);
        let (tx, rx) = async_channel::bounded(1);
        spawn_async_task(async move {
            blobs.add_slice(&bytes).await.expect("add slice");
            tx.send(()).await.expect("send");
        });
        rx.recv_blocking().expect("upload");
        hash
    }

    /// Commit the doc, then tick the app until `cond` returns true.
    /// Panics if the condition is not met within the timeout.
    pub fn tick_until<F: FnMut(&mut World) -> bool>(&mut self, mut cond: F) {
        self.doc.commit();
        for _ in 0..200 {
            self.app.update();
            if cond(self.app.world_mut()) {
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("tick_until condition not met within timeout");
    }
}

#[fixture]
pub fn ctx() -> TestContext {
    TestContext::default()
}

#[fixture]
pub fn ctx_physics() -> TestContext {
    TestContext::with_physics()
}

#[fixture]
pub fn ctx_wds() -> TestContext {
    TestContext::with_wds()
}

fn setup_blobs() -> Blobs {
    let (tx, rx) = async_channel::bounded(1);
    spawn_async_task(async move {
        let store = MemStore::default();
        let blobs = store.blobs().clone();
        tx.send(blobs).await.expect("send");
        // Keep MemStore alive — its background task drives blob queries.
        let _store = store;
        std::future::pending::<()>().await;
    });
    rx.recv_blocking().expect("setup blobs")
}
