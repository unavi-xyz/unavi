// Compiled into every integration-test binary in this crate, each of which
// only uses a subset of these helpers.
#![expect(dead_code)]

use std::{
    sync::{
        Arc,
        Mutex,
    },
    time::Duration,
};

use avian3d::PhysicsPlugins;
use bevy::{
    asset::AssetPlugin,
    ecs::schedule::{
        ScheduleLabel,
        SingleThreadedExecutor,
    },
    prelude::*,
    transform::TransformPlugin,
};
use bevy_hsd::attributes::material_graph::ShaderGraphMaterial;
use bevy_wds::{
    LocalBlobs,
    WdsPlugin,
};
use hsd::{
    attributes::Attribute,
    id::PrimId,
    state::SceneState,
};
use iroh_blobs::{
    api::blobs::Blobs,
    store::mem::MemStore,
};
use rstest::fixture;
use unavi_util::async_task::spawn_async_task;

pub struct TestContext {
    pub app:   App,
    pub state: Arc<Mutex<SceneState>>,
    blobs:     Option<Blobs>,
}

/// `#[traced_test]` installs a thread-local subscriber, so a warning emitted
/// from a system running on a worker thread is invisible to `logs_contain`.
/// Running the schedules on the calling thread keeps validation warnings
/// assertable.
fn run_on_test_thread(app: &mut App) {
    app.edit_schedule(Update.intern(), |schedule| {
        schedule.set_executor(SingleThreadedExecutor::default());
    });
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
        .init_asset::<StandardMaterial>()
        .init_asset::<Shader>()
        .init_asset::<ShaderGraphMaterial>();
        run_on_test_thread(&mut app);

        let mut ctx = Self {
            app,
            state: Arc::default(),
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
    /// and will panic on the placeholder MAX values if the collider path
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
        .init_asset::<Shader>()
        .init_asset::<ShaderGraphMaterial>()
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_secs_f32(1.0 / 60.0),
        ));
        run_on_test_thread(&mut app);
        app.finish();
        app.cleanup();

        let mut ctx = Self {
            app,
            state: Arc::default(),
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
        .init_asset::<Shader>()
        .init_asset::<ShaderGraphMaterial>()
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(10)));
        run_on_test_thread(&mut app);

        app.world_mut().spawn(LocalBlobs(blobs.clone()));

        let mut ctx = Self {
            app,
            state: Arc::default(),
            blobs: Some(blobs),
        };

        ctx.spawn_hsd();

        ctx
    }

    pub fn spawn_hsd(&mut self) {
        self.app
            .world_mut()
            .spawn(bevy_hsd::Hsd(Arc::clone(&self.state)));
    }

    fn with_state<T>(&self, f: impl FnOnce(&mut SceneState) -> T) -> T {
        f(&mut self.state.lock().expect("lock state"))
    }

    pub fn create_prim(&self) -> PrimId {
        self.with_state(|state| state.create_prim(None))
    }

    pub fn create_child(&self, parent: PrimId) -> PrimId {
        self.with_state(|state| state.create_prim(Some(parent)))
    }

    pub fn set_attr<A: Attribute>(&self, prim: PrimId, value: &A) {
        self.with_state(|state| state.set_attribute(prim, value).expect("set attribute"));
    }

    pub fn remove_attr<A: Attribute>(&self, prim: PrimId) {
        self.with_state(|state| state.remove_property(prim, A::KEY));
    }

    pub fn set_relationship(&self, prim: PrimId, name: &str, target: PrimId) {
        self.with_state(|state| {
            state
                .set_relationship(prim, name, target)
                .expect("set relationship");
        });
    }

    pub fn remove_property(&self, prim: PrimId, name: &str) {
        self.with_state(|state| state.remove_property(prim, name));
    }

    pub fn set_slot(&self, prim: PrimId, slot: &str, bytes: Vec<u8>) {
        self.with_state(|state| state.set_slot(prim, slot, bytes).expect("set slot"));
    }

    pub fn remove_slot(&self, prim: PrimId, slot: &str) {
        self.with_state(|state| state.remove_slot(prim, slot));
    }

    pub fn remove_prim(&self, prim: PrimId) {
        self.with_state(|state| state.remove_prim(prim));
    }

    /// Tick the app until `cond` returns true.
    /// Panics if the condition is not met within the timeout.
    pub fn tick_until<F: FnMut(&mut World) -> bool>(&mut self, mut cond: F) {
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
