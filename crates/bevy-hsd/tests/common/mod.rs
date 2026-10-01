// Shared across integration-test binaries; each uses only a subset.
#![expect(dead_code)]

use std::{
    sync::{
        Arc,
        Mutex,
    },
    time::Duration,
};

use bevy::{
    asset::AssetPlugin,
    ecs::schedule::{
        ScheduleLabel,
        SingleThreadedExecutor,
    },
    prelude::*,
    transform::TransformPlugin,
};
use bevy_hsd::attributes::shader::material::ShaderGraphMaterial;
use bevy_iroh::store::LocalBlobs;
use bevy_msdf::font::RegisterFont;
use hsd::{
    attributes::{
        collider::{
            ColliderIndices,
            ColliderVertices,
        },
        image::ImageData,
        mesh::{
            self,
            MeshIndices,
            MeshStream,
        },
        shader::{
            ShaderGraph,
            graph::{
                DisplacementGraph,
                SurfaceGraph,
                SurfaceOutput,
                UnlitOutput,
            },
            node::{
                Node,
                Port,
            },
            value::GraphValue,
        },
    },
    id::PrimId,
    property::{
        Payload,
        Property,
        name::PropName,
    },
    state::{
        HsdState,
        entry::Entry,
    },
};
use iroh::{
    Endpoint,
    SecretKey,
    endpoint::presets::N0DisableRelay,
};
use iroh_blobs::{
    Hash,
    api::blobs::Blobs,
    store::mem::MemStore,
};
use iroh_docs::{
    Author,
    NamespaceId,
};
use rstest::fixture;
use unavi_util::async_task::spawn_async_task;
use wds::{
    Store,
    builder::StoreBuilder,
    document::Document,
};

pub struct TestContext {
    pub app:   App,
    pub state: Arc<Mutex<HsdState>>,
    /// The entity holding [`Self::state`].
    pub doc:   Entity,
    blobs:     Option<Blobs>,
}

/// `#[traced_test]` installs a thread-local subscriber, so a warning from a
/// worker-thread system is invisible to `logs_contain`. Running schedules on
/// the calling thread keeps validation warnings assertable.
fn run_on_test_thread(app: &mut App) {
    app.edit_schedule(Update.intern(), |schedule| {
        schedule.set_executor(SingleThreadedExecutor::default());
    });
}

/// The client fetches its faces over iroh. A test app has no network, so text
/// would lay out against an empty chain and draw nothing.
fn register_font(app: &mut App) {
    app.world_mut()
        .trigger(RegisterFont(Arc::<[u8]>::from(notosans::REGULAR_TTF)));
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
        register_font(&mut app);

        let mut ctx = Self {
            app,
            state: Arc::default(),
            doc: Entity::PLACEHOLDER,
            blobs: None,
        };

        ctx.spawn_hsd();

        ctx
    }
}

impl TestContext {
    /// Same as `default()` with physics enabled. Needed by any test that
    /// exercises colliders or rigid bodies. Avian's `On<Add, Collider>`
    /// observer reads `Position`/`Rotation` and panics on the placeholder MAX
    /// values if a `Collider` is inserted without seeding them.
    pub fn with_physics() -> Self {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            TransformPlugin,
            bevy::scene::ScenePlugin,
            unavi_physics::PhysicsPlugin,
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
        register_font(&mut app);
        app.finish();
        app.cleanup();

        let mut ctx = Self {
            app,
            state: Arc::default(),
            doc: Entity::PLACEHOLDER,
            blobs: None,
        };

        ctx.spawn_hsd();

        ctx
    }

    pub fn with_blobs() -> Self {
        let blobs = setup_blobs();

        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            TransformPlugin,
            bevy_iroh::IrohPlugin,
            bevy_hsd::HsdPlugin,
        ))
        .init_asset::<Image>()
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .init_asset::<Shader>()
        .init_asset::<ShaderGraphMaterial>()
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(10)));
        run_on_test_thread(&mut app);
        register_font(&mut app);

        app.world_mut().spawn(LocalBlobs(blobs.clone()));

        let mut ctx = Self {
            app,
            state: Arc::default(),
            doc: Entity::PLACEHOLDER,
            blobs: Some(blobs),
        };

        ctx.spawn_hsd();

        ctx
    }

    pub fn spawn_hsd(&mut self) {
        self.doc = self
            .app
            .world_mut()
            .spawn(bevy_hsd::document::Hsd(Arc::clone(&self.state)))
            .id();
    }

    /// A second document in the same app, for anything asserting about what
    /// two documents share or hold separately.
    pub fn spawn_document(&mut self) -> TestDocument {
        let state = Arc::<Mutex<HsdState>>::default();
        let entity = self
            .app
            .world_mut()
            .spawn(bevy_hsd::document::Hsd(Arc::clone(&state)))
            .id();
        TestDocument { entity, state }
    }

    fn with_state<T>(&self, f: impl FnOnce(&mut HsdState) -> T) -> T {
        f(&mut self.state.lock().expect("lock state"))
    }

    pub fn despawn_document(&mut self, doc: &TestDocument) {
        self.app.world_mut().despawn(doc.entity);
    }

    pub fn create_prim(&self) -> PrimId {
        self.with_state(|state| state.create_prim(None))
    }

    pub fn create_child(&self, parent: PrimId) -> PrimId {
        self.with_state(|state| state.create_prim(Some(parent)))
    }

    /// The entity standing for `prim` in the scene, which the diff spawns on
    /// the tick after the write.
    pub fn prim_entity(&self, doc: Entity, prim: PrimId) -> Entity {
        self.app
            .world()
            .get::<bevy_hsd::prim::PrimIndex>(doc)
            .expect("document has a prim index")
            .get(prim)
            .expect("prim is in the scene")
    }

    pub fn set_attr<A: Property>(&self, prim: PrimId, value: &A) {
        self.with_state(|state| state.set_attribute(prim, value).expect("set attribute"));
    }

    pub fn remove_attr<A: Property>(&self, prim: PrimId) {
        self.with_state(|state| state.remove_property(prim, &A::NAME));
    }

    pub fn set_mesh_stream(&self, prim: PrimId, name: &str, bytes: Vec<u8>) {
        let field = mesh::stream(name).expect("valid stream name");
        self.with_state(|state| {
            state
                .set_payload(prim, &field, &MeshStream(bytes))
                .expect("set stream");
        });
    }

    pub fn set_mesh_indices(&self, prim: PrimId, bytes: Vec<u8>) {
        self.set_attr(prim, &MeshIndices(bytes));
    }

    pub fn set_image_data(&self, prim: PrimId, bytes: Vec<u8>) {
        self.set_attr(prim, &ImageData(bytes));
    }

    pub fn set_collider_vertices(&self, prim: PrimId, bytes: Vec<u8>) {
        self.set_attr(prim, &ColliderVertices(bytes));
    }

    pub fn set_collider_indices(&self, prim: PrimId, bytes: Vec<u8>) {
        self.set_attr(prim, &ColliderIndices(bytes));
    }

    pub fn set_shader_graph(&self, prim: PrimId, bytes: Vec<u8>) {
        let graph = ShaderGraph::decode(&bytes).expect("decode graph");
        self.set_attr(prim, &graph);
    }

    pub fn set_relationship(&self, prim: PrimId, name: &PropName, target: PrimId) {
        self.with_state(|state| {
            state
                .set_relationship(prim, name, target)
                .expect("set relationship");
        });
    }

    pub fn remove_property(&self, prim: PrimId, name: &PropName) {
        self.with_state(|state| state.remove_property(prim, name));
    }

    /// Tick the app until `cond` returns true. Panics if the timeout elapses
    /// first.
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

/// A document beside [`TestContext`]'s own, ticked by the same app.
pub struct TestDocument {
    pub entity: Entity,
    state:      Arc<Mutex<HsdState>>,
}

impl TestDocument {
    fn with_state<T>(&self, f: impl FnOnce(&mut HsdState) -> T) -> T {
        f(&mut self.state.lock().expect("lock state"))
    }

    pub fn create_prim(&self) -> PrimId {
        self.with_state(|state| state.create_prim(None))
    }

    pub fn set_attr<A: Property>(&self, prim: PrimId, value: &A) {
        self.with_state(|state| state.set_attribute(prim, value).expect("set attribute"));
    }

    pub fn set_shader_graph(&self, prim: PrimId, bytes: Vec<u8>) {
        let graph = ShaderGraph::decode(&bytes).expect("decode graph");
        self.set_attr(prim, &graph);
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
pub fn ctx_blobs() -> TestContext {
    TestContext::with_blobs()
}

fn setup_blobs() -> Blobs {
    let (tx, rx) = async_channel::bounded(1);
    spawn_async_task(async move {
        let store = MemStore::default();
        let blobs = store.blobs().clone();
        tx.send(blobs).await.expect("send");
        // Keep MemStore alive. Its background task drives blob queries.
        let _store = store;
        std::future::pending::<()>().await;
    });
    rx.recv_blocking().expect("setup blobs")
}

#[must_use]
pub const fn const_f(v: f32) -> Port {
    Port::Const(GraphValue::Float(v))
}

#[must_use]
pub const fn const_v2(v: [f32; 2]) -> Port {
    Port::Const(GraphValue::Vec2(v))
}

#[must_use]
pub const fn const_v3(v: [f32; 3]) -> Port {
    Port::Const(GraphValue::Vec3(v))
}

#[must_use]
pub const fn const_color(v: [f32; 4]) -> Port {
    Port::Const(GraphValue::Color(v))
}

#[must_use]
pub const fn node(i: u16) -> Port {
    Port::Node(i)
}

#[must_use]
pub const fn input(i: u16) -> Port {
    Port::Input(i)
}

/// An unlit surface output with the given color and no clip threshold.
#[must_use]
pub const fn unlit(color: Port) -> SurfaceOutput {
    SurfaceOutput::Unlit(UnlitOutput {
        color,
        alpha_clip_threshold: None,
    })
}

/// A graph whose surface network holds `nodes` and has a default unlit output.
#[must_use]
pub fn graph(nodes: Vec<Node>) -> ShaderGraph {
    ShaderGraph {
        surface: SurfaceGraph {
            nodes,
            output: SurfaceOutput::Unlit(UnlitOutput::default()),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[must_use]
pub fn graph_with_output(nodes: Vec<Node>, output: SurfaceOutput) -> ShaderGraph {
    ShaderGraph {
        surface: SurfaceGraph {
            nodes,
            output,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[must_use]
pub fn displaced(nodes: Vec<Node>, position_offset: Option<Port>) -> ShaderGraph {
    ShaderGraph {
        surface: SurfaceGraph::default(),
        displacement: Some(DisplacementGraph {
            nodes,
            position_offset,
            normal_override: None,
            world_position_offset: None,
        }),
        ..Default::default()
    }
}

/// A graph displacing in world space rather than local space.
#[must_use]
pub fn displaced_world(nodes: Vec<Node>, world_position_offset: Option<Port>) -> ShaderGraph {
    ShaderGraph {
        surface: SurfaceGraph::default(),
        displacement: Some(DisplacementGraph {
            nodes,
            position_offset: None,
            normal_override: None,
            world_position_offset,
        }),
        ..Default::default()
    }
}

/// Runs `future` on the task runtime the app's own tasks use.
fn block_on<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) -> T {
    let (tx, rx) = async_channel::bounded(1);
    spawn_async_task(async move {
        tx.send(future.await).await.expect("send result");
    });
    rx.recv_blocking().expect("task result")
}

/// An in-memory store, for tests whose entries go through iroh-docs.
pub struct Backing(Store);

impl Backing {
    pub fn new() -> Self {
        Self(block_on(async {
            let secret_key = SecretKey::generate();
            let author = Author::from_bytes(&secret_key.to_bytes());
            let endpoint = Endpoint::builder(N0DisableRelay)
                .secret_key(secret_key)
                .bind()
                .await
                .expect("bind endpoint");
            StoreBuilder::new(endpoint, author)
                .build()
                .await
                .expect("build store")
                .store
        }))
    }

    pub fn document(&self) -> Document {
        let store = self.0.clone();
        block_on(async move { store.create().await.expect("create document") })
    }

    /// A handle to the underlying store, for a test that needs to spawn its
    /// own `LocalStore`.
    pub fn store(&self) -> Store {
        self.0.clone()
    }

    pub fn namespaces(&self) -> Vec<NamespaceId> {
        let store = self.0.clone();
        block_on(async move {
            store
                .list()
                .await
                .expect("list namespaces")
                .into_iter()
                .map(|(ns, _)| ns)
                .collect()
        })
    }

    /// Whether `ns` is in the sync set, answering peers that request it.
    pub fn is_served(&self, ns: NamespaceId) -> bool {
        let store = self.0.clone();
        block_on(async move {
            let doc = store.open(ns).await.expect("open namespace");
            doc.status().await.expect("namespace status").sync
        })
    }

    /// Stores `bytes` as a blob without pointing any key at it.
    pub fn add_bytes(&self, bytes: Vec<u8>) -> Hash {
        let store = self.0.clone();
        block_on(async move {
            store
                .blobs()
                .add_bytes(bytes)
                .await
                .expect("add bytes")
                .hash
        })
    }
}

pub fn set_entry(doc: &Document, entry: &Entry) {
    let (doc, entry) = (doc.clone(), entry.clone());
    block_on(async move {
        doc.set(entry.key, entry.value).await.expect("set entry");
    });
}

pub fn remove_key(doc: &Document, key: String) {
    let doc = doc.clone();
    block_on(async move {
        doc.remove(key).await.expect("remove key");
    });
}

pub fn set_hash(doc: &Document, key: String, hash: Hash, size: u64) {
    let doc = doc.clone();
    block_on(async move {
        doc.set_hash(key, hash, size).await.expect("set hash");
    });
}
