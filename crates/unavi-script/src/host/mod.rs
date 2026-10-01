//! The engine-agnostic implementation of every host call.
//!
//! Each `wired:*` package has a module here taking and returning host types.
//! Permission and ownership checks happen here, once, so a binding only
//! lowers values and can never forget a check.

use std::sync::{
    Arc,
    Mutex,
};

use bevy::{
    ecs::system::SystemParam,
    platform::collections::HashSet,
    prelude::*,
};
use bevy_async::AsyncWorld;
use hsd::{
    id::{
        DocId,
        PrimId,
    },
    state::HsdState,
};
use iroh_docs::NamespaceId;
use unavi_policy::{
    permissions::HostApi,
    quota::Quota,
};
use unavi_space::authority::SpaceView;

use crate::{
    ScriptSystems,
    error::ScriptError,
    host::{
        event::MessageSubscription,
        handles::HandleTable,
        input::InputSubscription,
        portal::IntentSubscription,
        scene::DocumentRes,
        shared_state::{
            agents::LocalAgentNodes,
            event_bus::EventBus,
            input_listeners::InputListeners,
            link_intents::LinkIntents,
            pointers::Pointers,
            transforms::TransformSnapshots,
        },
        storage::{
            PendingEntries,
            PendingValue,
        },
    },
};

pub mod agent;
pub mod event;
pub mod handles;
pub mod input;
pub mod peer;
pub mod physics;
pub mod portal;
pub mod queue;
pub mod scene;
pub mod shading;
pub mod shared_state;
pub mod storage;

/// The cross-script state every [`ScriptHost`] holds a clone of.
#[derive(Clone)]
pub struct SharedState {
    pub agents:          LocalAgentNodes,
    pub pointers:        Pointers,
    pub transforms:      TransformSnapshots,
    pub event_bus:       EventBus,
    pub input_listeners: InputListeners,
    pub link_intents:    LinkIntents,
}

/// Reads the [`SharedState`] resources.
#[derive(SystemParam)]
pub struct SharedResources<'w> {
    agents:          Res<'w, LocalAgentNodes>,
    pointers:        Res<'w, Pointers>,
    transforms:      Res<'w, TransformSnapshots>,
    event_bus:       Res<'w, EventBus>,
    input_listeners: Res<'w, InputListeners>,
    link_intents:    Res<'w, LinkIntents>,
}

impl SharedResources<'_> {
    #[must_use]
    pub fn get(&self) -> SharedState {
        SharedState {
            agents:          self.agents.clone(),
            pointers:        self.pointers.clone(),
            transforms:      self.transforms.clone(),
            event_bus:       self.event_bus.clone(),
            input_listeners: self.input_listeners.clone(),
            link_intents:    self.link_intents.clone(),
        }
    }
}

/// Who a script is, and what it holds.
pub struct ScriptIdentity {
    pub doc:      DocId,
    pub prim:     PrimId,
    pub state:    Arc<Mutex<HsdState>>,
    pub view:     SpaceView,
    pub quota:    Arc<Quota>,
    pub world:    AsyncWorld,
    /// This node's root document, or `None` when it runs without a store.
    pub root_doc: Option<NamespaceId>,
}

/// One script's host context: what it is, and every handle it holds.
///
/// Dropping it drops every handle, which closes the script's listeners and
/// subscriptions; nothing it opened outlives it.
pub struct ScriptHost {
    pub(crate) doc:             DocId,
    pub(crate) prim:            PrimId,
    pub(crate) state:           Arc<Mutex<HsdState>>,
    pub(crate) view:            SpaceView,
    pub(crate) quota:           Arc<Quota>,
    pub(crate) world:           AsyncWorld,
    pub(crate) root_doc:        Option<NamespaceId>,
    pub(crate) shared:          SharedState,
    pub(crate) documents:       HandleTable<DocumentRes>,
    pub(crate) messages:        HandleTable<MessageSubscription>,
    pub(crate) inputs:          HandleTable<InputSubscription>,
    pub(crate) intents:         HandleTable<IntentSubscription>,
    pub(crate) pending_values:  HandleTable<PendingValue>,
    pub(crate) pending_entries: HandleTable<PendingEntries>,
    /// Documents this script created and has not deleted.
    pub(crate) minted:          HashSet<DocId>,
    /// The host clock at the start of the current tick, in seconds.
    pub(crate) now:             f64,
}

impl ScriptHost {
    #[must_use]
    pub fn new(identity: ScriptIdentity, shared: SharedState) -> Self {
        Self {
            doc: identity.doc,
            prim: identity.prim,
            state: identity.state,
            view: identity.view,
            quota: identity.quota,
            world: identity.world,
            root_doc: identity.root_doc,
            shared,
            documents: HandleTable::default(),
            messages: HandleTable::default(),
            inputs: HandleTable::default(),
            intents: HandleTable::default(),
            pending_values: HandleTable::default(),
            pending_entries: HandleTable::default(),
            minted: HashSet::default(),
            now: 0.0,
        }
    }

    /// Refuses a call the script's document is not trusted for.
    ///
    /// Read per call rather than captured at instantiation: an author can
    /// resolve later, and the user can withdraw a grant.
    pub fn require(&self, api: HostApi) -> Result<(), ScriptError> {
        Ok(self.view.permissions(self.doc).require(api)?)
    }

    /// Whether the script's document currently holds `api`.
    #[must_use]
    pub fn granted(&self, api: HostApi) -> bool {
        self.require(api).is_ok()
    }

    /// Whether this script owns `doc`: its own document, one it created, or
    /// one whose author also authors its own. Only an owned document may be
    /// written, placed, listened to or moved.
    ///
    /// Authorship is read per call, so a document whose author resolves late
    /// is judged by what is known now.
    #[must_use]
    pub fn owns(&self, doc: DocId) -> bool {
        doc == self.doc
            || self.minted.contains(&doc)
            || self
                .view
                .author(self.doc)
                .is_some_and(|author| self.view.author(doc).as_ref() == Some(&author))
    }

    /// The document behind `handle`.
    pub fn document(&self, handle: u32) -> Result<&DocumentRes, ScriptError> {
        self.documents.get(handle)
    }

    /// The document behind `handle`, if the script owns it.
    pub fn owned_document(&self, handle: u32) -> Result<&DocumentRes, ScriptError> {
        let doc = self.documents.get(handle)?;
        if self.owns(doc.id) {
            Ok(doc)
        } else {
            Err(ScriptError::Forbidden)
        }
    }

    /// Runs `f` against the world at the next pump, answering what it
    /// returns.
    pub async fn world_call<T, F>(&self, f: F) -> Result<T, ScriptError>
    where
        T: Send + 'static,
        F: FnOnce(&mut World) -> T + Send + 'static,
    {
        self.world
            .commands()
            .send_with(f)
            .await
            .ok_or_else(|| ScriptError::Internal("the world is gone".into()))
    }

    /// Holds every document this script can write open for one tick, so a
    /// tick suspended between two host calls never has half its writes
    /// drawn. Documents opened during the tick are not covered: the guard
    /// closes only what it opened.
    ///
    /// `now` is the host clock, which messages sent during the tick are
    /// stamped with.
    #[must_use]
    pub fn open_tick(&mut self, now: f64) -> TickGuard {
        self.now = now;
        let mut held = vec![Arc::clone(&self.state)];
        held.extend(self.documents.values().map(|doc| Arc::clone(&doc.state)));
        for state in &held {
            if let Ok(mut state) = state.lock() {
                state.open_tick();
            }
        }
        TickGuard(held)
    }
}

/// Closes the write boundaries [`ScriptHost::open_tick`] opened, including
/// when the tick trapped or was cut short.
pub struct TickGuard(Vec<Arc<Mutex<HsdState>>>);

impl Drop for TickGuard {
    fn drop(&mut self) {
        for state in &self.0 {
            if let Ok(mut state) = state.lock() {
                state.close_tick();
            }
        }
    }
}

/// The shared state and the systems that keep it current.
pub struct HostPlugin;

impl Plugin for HostPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<input::bridge::Pressing>()
            .init_resource::<LocalAgentNodes>()
            .init_resource::<EventBus>()
            .init_resource::<InputListeners>()
            .init_resource::<LinkIntents>()
            .init_resource::<Pointers>()
            .init_resource::<TransformSnapshots>()
            .add_observer(input::bridge::bridge_press)
            .add_observer(input::bridge::bridge_enter)
            .add_observer(input::bridge::bridge_leave)
            .add_observer(input::bridge::bridge_scroll)
            .add_observer(shared_state::agents::forget_local_agent)
            .add_observer(shared_state::transforms::register_nodes)
            .add_observer(shared_state::transforms::deregister_transforms)
            .add_observer(shared_state::transforms::deregister_doc_root)
            .add_systems(
                Update,
                (
                    shared_state::agents::track_local_agent,
                    shared_state::pointers::snapshot_pointers,
                    input::bridge::bridge_buttons,
                    input::bridge::bridge_device_scroll
                        .run_if(unavi_input::capture::scene_has_input),
                    input::bridge::bridge_menu,
                ),
            )
            .add_systems(
                Update,
                (
                    (
                        bevy::transform::systems::mark_dirty_trees,
                        bevy::transform::systems::propagate_parent_transforms,
                        bevy::transform::systems::sync_simple_transforms,
                    )
                        .chain(),
                    shared_state::transforms::snapshot_transforms,
                    shared_state::transforms::snapshot_doc_roots,
                )
                    .chain()
                    .in_set(ScriptSystems::Snapshot),
            )
            .add_systems(
                PostUpdate,
                (
                    shared_state::transforms::snapshot_transforms,
                    shared_state::transforms::snapshot_doc_roots,
                )
                    .after(bevy::transform::TransformSystems::Propagate),
            )
            .add_systems(FixedUpdate, crate::link_intents::offer_link_intents);
    }
}
