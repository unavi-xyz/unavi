//! `wired:input`: pointer input, aimed at prims or for the whole device.

use std::sync::Arc;

use bevy::{
    ecs::system::SystemParam,
    prelude::*,
};
use bevy_hsd::{
    document::HsdDocId,
    prim::{
        Prim,
        PrimOf,
    },
};
use hsd::id::{
    DocId,
    PrimId,
};
use unavi_input::pointer::{
    PointerHit,
    PointerKind,
};
use unavi_policy::{
    permissions::HostApi,
    quota::{
        Stock,
        StockLease,
    },
};

use crate::{
    error::ScriptError,
    host::{
        ScriptHost,
        queue::Queue,
        shared_state::input_listeners::{
            InputListeners,
            Target,
        },
    },
};

pub mod bridge;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub origin:    Vec3,
    pub direction: Vec3,
}

impl From<Ray3d> for Ray {
    fn from(ray: Ray3d) -> Self {
        Self {
            origin:    ray.origin,
            direction: *ray.direction,
        }
    }
}

/// Where a pointer's ray met a collider.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub target:   Option<(DocId, PrimId)>,
    pub position: Vec3,
    pub normal:   Vec3,
    pub distance: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Button {
    Trigger,
    Grip,
    Menu,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Pressed(Button),
    Released(Button),
    Scroll(Vec2),
    Entered,
    Left,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputEvent {
    pub pointer: PointerKind,
    pub action:  Action,
    pub ray:     Ray,
    pub hit:     Option<Hit>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pointer {
    pub kind:    PointerKind,
    pub active:  bool,
    pub ray:     Ray,
    pub trigger: f32,
    pub grip:    f32,
    pub axis:    Vec2,
    pub hit:     Option<Hit>,
}

impl Pointer {
    /// A pointer the rig never spawned: no hands on desktop, no screen
    /// pointer in VR. Still listed, so a script can see it is not there.
    #[must_use]
    pub const fn inactive(kind: PointerKind) -> Self {
        Self {
            kind,
            active: false,
            ray: Ray {
                origin:    Vec3::ZERO,
                direction: Vec3::NEG_Z,
            },
            trigger: 0.0,
            grip: 0.0,
            axis: Vec2::ZERO,
            hit: None,
        }
    }
}

/// Which prim of which document an entity is.
#[derive(SystemParam)]
pub struct PrimTargets<'w, 's> {
    prims: Query<'w, 's, (&'static Prim, &'static PrimOf)>,
    docs:  Query<'w, 's, &'static HsdDocId>,
}

impl PrimTargets<'_, '_> {
    #[must_use]
    pub fn of(&self, entity: Entity) -> Option<(DocId, PrimId)> {
        let (prim, doc) = self.prims.get(entity).ok()?;
        Some((self.docs.get(doc.0).ok()?.0, prim.0))
    }

    #[must_use]
    pub fn hit(&self, hit: PointerHit) -> Hit {
        Hit {
            target:   self.of(hit.entity),
            position: hit.position,
            normal:   hit.normal,
            distance: hit.distance,
        }
    }
}

/// A guest's open input subscription. Closes its listener when dropped.
pub struct InputSubscription {
    id:        u64,
    listeners: InputListeners,
    queue:     Arc<Queue<InputEvent>>,
    _lease:    StockLease,
}

impl InputSubscription {
    fn drain(&self, max: u32) -> Vec<InputEvent> {
        self.queue.drain(max)
    }

    fn dropped(&self) -> u64 {
        self.queue.dropped()
    }
}

impl Drop for InputSubscription {
    fn drop(&mut self) {
        self.listeners.close(self.id);
    }
}

/// Input aimed at `prim` of a document the script owns, or anything beneath
/// it.
pub fn listen_targeted(host: &mut ScriptHost, doc: u32, prim: PrimId) -> Result<u32, ScriptError> {
    host.require(HostApi::Input)?;
    let doc = host.owned_document(doc)?.id;
    subscribe(host, Target::Prim(doc, prim))
}

/// Every input of the local user's devices.
pub fn listen_device(host: &mut ScriptHost) -> Result<u32, ScriptError> {
    host.require(HostApi::InputContext)?;
    subscribe(host, Target::Device)
}

/// A subscription refused a handle drops at once, closing its listener.
fn subscribe(host: &mut ScriptHost, target: Target) -> Result<u32, ScriptError> {
    let lease = host.quota.lease(Stock::Receptors, 1)?;
    let queue = Arc::new(Queue::default());
    let listeners = host.shared.input_listeners.clone();
    let id = listeners.open(target, Arc::clone(&queue));
    let subscription = InputSubscription {
        id,
        listeners,
        queue,
        _lease: lease,
    };
    Ok(host.inputs.insert(subscription, &host.quota)?)
}

pub fn pointers(host: &ScriptHost) -> Result<Vec<Pointer>, ScriptError> {
    host.require(HostApi::InputContext)?;
    Ok(host.shared.pointers.all())
}

pub fn drain(
    host: &ScriptHost,
    subscription: u32,
    max: u32,
) -> Result<Vec<InputEvent>, ScriptError> {
    Ok(host.inputs.get(subscription)?.drain(max))
}

pub fn dropped(host: &ScriptHost, subscription: u32) -> Result<u64, ScriptError> {
    Ok(host.inputs.get(subscription)?.dropped())
}

pub fn close(host: &mut ScriptHost, subscription: u32) -> Result<(), ScriptError> {
    host.inputs.remove(subscription).map(drop)
}
