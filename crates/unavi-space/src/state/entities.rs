use bevy::prelude::*;
use bevy_hsd::HsdNamespace;
use iroh::EndpointId;
use iroh_docs::NamespaceId;
use unavi_policy::{
    registry::Policy,
    space::Space,
};
use unavi_util::async_commands::AsyncCommands;

use crate::state::{
    message::StateMsg,
    replicas::{
        self,
        KvError,
    },
};

#[derive(Component)]
#[relationship(relationship_target = DocStates)]
pub struct StateDoc(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = StateDoc, linked_spawn)]
pub struct DocStates(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = PeerStates)]
pub struct StatePeer(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = StatePeer, linked_spawn)]
pub struct PeerStates(Vec<Entity>);

/// The local peer's state owner, spawned once and living for the session. Local
/// state is cleaned only when its document despawns (leaving a space).
#[derive(Component)]
pub struct LocalPeer;

/// A connected remote peer's inbound-state owner, tied to its state stream.
/// Despawning it on disconnect cascades away all of that peer's state.
#[derive(Component)]
pub struct RemotePeer(pub EndpointId);

/// Generation of the state stream feeding a [`RemotePeer`] entity.
///
/// A newer stream (a canonical connection replacing a racing duplicate) takes
/// the entity over, so the superseded stream's teardown leaves it intact.
#[derive(Component)]
pub struct StreamGen(u64);

/// Claims the state entity for `peer`, reusing an existing one so overlapping
/// streams for the same peer never hold duplicate store guards.
pub fn claim_remote_peer(world: &mut World, peer: EndpointId, generation: u64) -> Entity {
    if let Some(e) = entity_by::<RemotePeer, _>(world, |r| r.0 == peer) {
        let newer = world.get::<StreamGen>(e).is_none_or(|g| generation > g.0);
        if newer {
            world.entity_mut(e).insert(StreamGen(generation));
        }
        return e;
    }
    world.spawn((RemotePeer(peer), StreamGen(generation))).id()
}

/// Despawns the peer's state entity only if `generation` still owns it.
pub fn release_remote_peer(world: &mut World, peer_ent: Entity, generation: u64) {
    if world
        .get::<StreamGen>(peer_ent)
        .is_some_and(|g| g.0 == generation)
    {
        world.despawn(peer_ent);
    }
}

/// A document tracked because some peer references it, anchoring its state
/// entities. Parented under the [`Space`] so leaving the space cascades it
/// away; unparented until the space is entered and adopts it.
#[derive(Component)]
pub struct SpaceDoc {
    pub doc:   NamespaceId,
    pub space: NamespaceId,
}

#[derive(Component)]
pub struct PinState {
    peer:   EndpointId,
    doc:    NamespaceId,
    local:  bool,
    /// Held because releasing a pin can hand ownership to another peer, which
    /// re-attributes the document quota, and a drop takes no arguments.
    policy: Policy,
}

impl PinState {
    fn register(
        policy: Policy,
        peer: EndpointId,
        doc: NamespaceId,
        space: NamespaceId,
        at: u64,
        local: bool,
    ) -> Option<Self> {
        if !replicas::add_pin(&policy, peer, doc, space, at) {
            return None;
        }
        if local {
            replicas::broadcast(&StateMsg::Pin { doc, space, at });
        }
        Some(Self {
            peer,
            doc,
            local,
            policy,
        })
    }
}

impl Drop for PinState {
    fn drop(&mut self) {
        replicas::remove_pin(&self.policy, self.peer, self.doc);
        if self.local {
            replicas::broadcast(&StateMsg::Unpin { doc: self.doc });
        }
    }
}

#[derive(Component)]
pub struct AuthorityState {
    peer:  EndpointId,
    doc:   NamespaceId,
    local: bool,
}

impl AuthorityState {
    fn apply(
        policy: &Policy,
        peer: EndpointId,
        doc: NamespaceId,
        space: NamespaceId,
        at: u64,
        local: bool,
    ) -> bool {
        let ok = replicas::add_authority(policy, peer, doc, space, at);
        if ok && local {
            replicas::broadcast(&StateMsg::Authority { doc, space, at });
        }
        ok
    }

    fn register(
        policy: &Policy,
        peer: EndpointId,
        doc: NamespaceId,
        space: NamespaceId,
        at: u64,
        local: bool,
    ) -> Option<Self> {
        Self::apply(policy, peer, doc, space, at, local).then_some(Self { peer, doc, local })
    }
}

impl Drop for AuthorityState {
    fn drop(&mut self) {
        replicas::remove_authority(self.peer, self.doc);
        if self.local {
            replicas::broadcast(&StateMsg::Unclaim { doc: self.doc });
        }
    }
}

/// Holds one KV cell for as long as the document anchoring it lives.
#[derive(Component)]
pub struct KvState {
    doc: NamespaceId,
    key: String,
}

impl Drop for KvState {
    fn drop(&mut self) {
        // No `KvForget` goes out. A cell belongs to the document, so it outlives
        // the peer that wrote it; an explicit delete propagates as a tombstone
        // at write time instead.
        replicas::remove_kv(self.doc, &self.key);
    }
}

fn entity_by<C: Component, F: Fn(&C) -> bool>(world: &mut World, pred: F) -> Option<Entity> {
    let mut query = world.query::<(Entity, &C)>();
    query.iter(world).find(|(_, c)| pred(c)).map(|(e, _)| e)
}

fn space_entity(world: &mut World, space: NamespaceId) -> Option<Entity> {
    entity_by::<Space, _>(world, |s| s.0 == space)
}

/// Resolves the entity anchoring `doc`, spawning a [`SpaceDoc`] tracker when
/// nothing exists yet. Trackers for spaces not yet entered stay unparented so
/// their state is kept until the space is joined.
fn doc_anchor(world: &mut World, doc: NamespaceId, space: NamespaceId) -> Entity {
    if let Some(e) = space_entity(world, doc) {
        return e;
    }
    if let Some(e) = entity_by::<HsdNamespace, _>(world, |r| r.0 == doc) {
        return e;
    }
    if let Some(e) = entity_by::<SpaceDoc, _>(world, |d| d.doc == doc) {
        return e;
    }
    let space_ent = space_entity(world, space);
    let tracker = world.spawn(SpaceDoc { doc, space }).id();
    if let Some(space_ent) = space_ent {
        world.entity_mut(tracker).insert(ChildOf(space_ent));
    }
    tracker
}

fn local_peer_entity(world: &mut World) -> Entity {
    if let Some(e) = entity_by::<LocalPeer, _>(world, |_| true) {
        return e;
    }
    world.spawn(LocalPeer).id()
}

/// Finds an existing state entity of `C` owned by `peer_ent` matching `pred`.
fn find_state<C: Component, F: Fn(&C) -> bool>(
    world: &World,
    peer_ent: Entity,
    pred: F,
) -> Option<Entity> {
    world
        .get::<PeerStates>(peer_ent)?
        .iter()
        .find(|e| world.get::<C>(*e).is_some_and(&pred))
}

/// Finds the doc-anchored cell guard for `key`, if one exists.
fn find_kv(world: &World, anchor: Entity, doc: NamespaceId, key: &str) -> Option<Entity> {
    world.get::<DocStates>(anchor)?.iter().find(|e| {
        world
            .get::<KvState>(*e)
            .is_some_and(|c| c.doc == doc && c.key == key)
    })
}

fn spawn_pin(
    world: &mut World,
    peer_ent: Entity,
    peer: EndpointId,
    doc: NamespaceId,
    space: NamespaceId,
    at: u64,
    local: bool,
) -> bool {
    if find_state::<PinState, _>(world, peer_ent, |p| p.doc == doc).is_some() {
        return true;
    }
    let anchor = doc_anchor(world, doc, space);
    let policy = world.resource::<Policy>().clone();
    let Some(state) = PinState::register(policy, peer, doc, space, at, local) else {
        warn!("pin on doc {doc} refused by quota");
        return false;
    };
    world.spawn((state, StateDoc(anchor), StatePeer(peer_ent)));
    true
}

fn spawn_authority(
    world: &mut World,
    peer_ent: Entity,
    peer: EndpointId,
    doc: NamespaceId,
    space: NamespaceId,
    at: u64,
    local: bool,
) {
    if find_state::<AuthorityState, _>(world, peer_ent, |a| a.doc == doc).is_some() {
        AuthorityState::apply(
            &world.resource::<Policy>().clone(),
            peer,
            doc,
            space,
            at,
            local,
        );
        return;
    }
    let anchor = doc_anchor(world, doc, space);
    let policy = world.resource::<Policy>().clone();
    let Some(state) = AuthorityState::register(&policy, peer, doc, space, at, local) else {
        return;
    };
    world.spawn((state, StateDoc(anchor), StatePeer(peer_ent)));
}

fn clear_authority(world: &mut World, peer_ent: Entity, doc: NamespaceId) {
    if let Some(e) = find_state::<AuthorityState, _>(world, peer_ent, |a| a.doc == doc) {
        world.despawn(e);
    }
}

fn clear_pin(world: &mut World, peer_ent: Entity, doc: NamespaceId) {
    if let Some(e) = find_state::<PinState, _>(world, peer_ent, |p| p.doc == doc) {
        world.despawn(e);
    }
}

fn set_kv(
    world: &mut World,
    peer: EndpointId,
    doc: NamespaceId,
    space: NamespaceId,
    key: String,
    value: Option<Vec<u8>>,
    at: u64,
    local: bool,
) -> Result<(), KvError> {
    if key.len() > replicas::KV_KEY_MAX_BYTES {
        return Err(KvError::KeyTooLong);
    }
    let anchor = doc_anchor(world, doc, space);
    let policy = world.resource::<Policy>().clone();
    replicas::add_kv(&policy, peer, doc, space, key.clone(), value.clone(), at)?;
    if local {
        replicas::broadcast(&StateMsg::Kv {
            doc,
            space,
            key: key.clone(),
            value,
            at,
        });
    }

    // Anchored to the document alone, so a disconnect leaves the cell intact
    // and a change of owner does not move it. One guard per key, whoever wrote
    // it last.
    if find_kv(world, anchor, doc, &key).is_none() {
        world.spawn((KvState { doc, key }, StateDoc(anchor)));
    }
    Ok(())
}

pub async fn self_pin(space: NamespaceId, doc: NamespaceId) -> bool {
    let Some(me) = crate::peer::self_peer_id() else {
        return false;
    };
    let at = replicas::current_millis();
    AsyncCommands::default()
        .send_with(move |world: &mut World| {
            let peer_ent = local_peer_entity(world);
            spawn_pin(world, peer_ent, me, doc, space, at, true)
        })
        .await
        .unwrap_or(false)
}

pub fn claim_authority(space: NamespaceId, doc: NamespaceId) {
    let Some(me) = crate::peer::self_peer_id() else {
        return;
    };
    let at = replicas::current_millis();
    let _ = AsyncCommands::default()
        .push(move |world: &mut World| {
            let peer_ent = local_peer_entity(world);
            spawn_authority(world, peer_ent, me, doc, space, at, true);
        })
        .try_send();
}

pub fn release_authority(doc: NamespaceId) {
    let _ = AsyncCommands::default()
        .push(move |world: &mut World| {
            if let Some(peer_ent) = entity_by::<LocalPeer, _>(world, |_| true) {
                clear_authority(world, peer_ent, doc);
            }
        })
        .try_send();
}

pub async fn doc_kv_set(
    space: NamespaceId,
    doc: NamespaceId,
    key: String,
    value: Vec<u8>,
) -> Result<(), KvError> {
    let Some(me) = crate::peer::self_peer_id() else {
        return Err(KvError::Other);
    };
    let at = replicas::current_millis();
    AsyncCommands::default()
        .send_with(move |world: &mut World| {
            set_kv(world, me, doc, space, key, Some(value), at, true)
        })
        .await
        .unwrap_or(Err(KvError::Other))
}

pub async fn doc_kv_delete(
    space: NamespaceId,
    doc: NamespaceId,
    key: String,
) -> Result<(), KvError> {
    let Some(me) = crate::peer::self_peer_id() else {
        return Err(KvError::Other);
    };
    let at = replicas::current_millis();
    AsyncCommands::default()
        .send_with(move |world: &mut World| set_kv(world, me, doc, space, key, None, at, true))
        .await
        .unwrap_or(Err(KvError::Other))
}

/// Applies a remote peer's delta under `peer_ent`. Called per message from the
/// network recv task; runs in the ECS via [`AsyncCommands`].
pub fn apply_remote(peer_ent: Entity, peer: EndpointId, msg: StateMsg) {
    if AsyncCommands::default()
        .push(move |world: &mut World| apply_in_world(world, peer_ent, peer, msg))
        .try_send()
        .is_err()
    {
        warn!("remote state delta dropped: command queue full");
    }
}

fn apply_in_world(world: &mut World, peer_ent: Entity, peer: EndpointId, msg: StateMsg) {
    match msg {
        StateMsg::Snapshot(snaps) => {
            let existing = world
                .get::<PeerStates>(peer_ent)
                .map(|s| s.iter().collect::<Vec<_>>())
                .unwrap_or_default();
            for e in existing {
                world.despawn(e);
            }
            for s in snaps {
                if let Some(at) = s.pin.filter(|at| replicas::time_valid(*at)) {
                    spawn_pin(world, peer_ent, peer, s.doc, s.space, at, false);
                }
                if let Some(at) = s.authority.filter(|at| replicas::time_valid(*at)) {
                    spawn_authority(world, peer_ent, peer, s.doc, s.space, at, false);
                }
                for kv in s.kv {
                    if replicas::time_valid(kv.at) {
                        let _ = set_kv(world, peer, s.doc, s.space, kv.key, kv.value, kv.at, false);
                    }
                }
            }
        }
        StateMsg::Pin { doc, space, at } if replicas::time_valid(at) => {
            spawn_pin(world, peer_ent, peer, doc, space, at, false);
        }
        StateMsg::Unpin { doc } => clear_pin(world, peer_ent, doc),
        StateMsg::Authority { doc, space, at } if replicas::time_valid(at) => {
            spawn_authority(world, peer_ent, peer, doc, space, at, false);
        }
        StateMsg::Unclaim { doc } => clear_authority(world, peer_ent, doc),
        StateMsg::Kv {
            doc,
            space,
            key,
            value,
            at,
        } if replicas::time_valid(at) => {
            let _ = set_kv(world, peer, doc, space, key, value, at, false);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        peer::set_self_peer_id,
        state::replicas::{
            TEST_LOCK,
            reset,
        },
    };

    fn h(seed: &[u8]) -> NamespaceId {
        NamespaceId::from(blake3::hash(seed).as_bytes())
    }

    /// A distinct, valid endpoint id per seed. Arbitrary bytes are not a curve
    /// point, so a key has to be derived rather than written down.
    fn peer(seed: u8) -> EndpointId {
        iroh::SecretKey::from_bytes(&[seed; 32]).public()
    }

    #[test]
    fn pin_guard_broadcasts_and_releases() {
        let _g = TEST_LOCK.lock();
        reset();
        let policy = Policy::new();
        let me = peer(1);
        set_self_peer_id(me);
        let space = h(b"pin-guard-space");
        let doc = h(b"pin-guard-doc");

        let (token, rx) = replicas::register_stream();
        assert!(matches!(rx.try_recv(), Ok(StateMsg::Snapshot(_))));

        let pin = PinState::register(policy, me, doc, space, 1, true).expect("pin registers");
        assert_eq!(replicas::owner(space, doc), Some(me));
        assert!(matches!(rx.try_recv(), Ok(StateMsg::Pin { doc: d, .. }) if d == doc));

        drop(pin);
        assert_eq!(replicas::owner(space, doc), None);
        assert!(matches!(rx.try_recv(), Ok(StateMsg::Unpin { doc: d }) if d == doc));

        replicas::unregister_stream(token);
        reset();
    }

    #[test]
    fn neutral_kv_guard_drop_does_not_forget() {
        let _g = TEST_LOCK.lock();
        reset();
        let me = peer(1);
        set_self_peer_id(me);
        let space = h(b"neutral-guard-space");

        let (token, rx) = replicas::register_stream();
        let _ = rx.try_recv();

        let mut world = World::new();
        world.init_resource::<Policy>();
        let space_ent = world.spawn(Space(space)).id();
        set_kv(
            &mut world,
            me,
            space,
            space,
            "k".into(),
            Some(b"v".to_vec()),
            1,
            true,
        )
        .expect("kv set");
        assert!(matches!(rx.try_recv(), Ok(StateMsg::Kv { .. })));

        // Tearing down the doc drops the cell locally but sends no retract, so
        // peers still holding it keep theirs.
        world.despawn(space_ent);
        assert_eq!(replicas::doc_kv_get(space, space, "k"), None);
        assert!(rx.try_recv().is_err());

        replicas::unregister_stream(token);
        reset();
    }

    #[test]
    fn neutral_kv_survives_writer_disconnect() {
        let _g = TEST_LOCK.lock();
        reset();
        let me = peer(1);
        set_self_peer_id(me);
        let remote = peer(3);
        let space = h(b"neutral-survive-space");

        let mut world = World::new();
        world.init_resource::<Policy>();
        let space_ent = world.spawn(Space(space)).id();
        let peer_ent = world.spawn(RemotePeer(remote)).id();
        set_kv(
            &mut world,
            remote,
            space,
            space,
            "k".into(),
            Some(b"v".to_vec()),
            1,
            false,
        )
        .expect("kv set");
        assert_eq!(replicas::doc_kv_get(space, space, "k"), Some(b"v".to_vec()));

        world.despawn(peer_ent);
        assert_eq!(
            replicas::doc_kv_get(space, space, "k"),
            Some(b"v".to_vec()),
            "space-owned kv should persist after the writer disconnects"
        );

        // The doc anchor still owns the cell's lifetime.
        world.despawn(space_ent);
        assert_eq!(replicas::doc_kv_get(space, space, "k"), None);
        reset();
    }

    #[test]
    fn despawning_doc_cascades_state_and_clears_store() {
        let _g = TEST_LOCK.lock();
        reset();
        let policy = Policy::new();
        let me = peer(1);
        set_self_peer_id(me);
        let space = h(b"cascade-doc-space");
        let doc = h(b"cascade-doc-doc");

        let mut world = World::new();
        world.init_resource::<Policy>();
        let peer_ent = world.spawn(LocalPeer).id();
        let doc_ent = world.spawn(SpaceDoc { doc, space }).id();
        let pin = PinState::register(policy, me, doc, space, 1, false).expect("pin");
        world.spawn((pin, StateDoc(doc_ent), StatePeer(peer_ent)));
        assert_eq!(replicas::owner(space, doc), Some(me));

        world.despawn(doc_ent);
        assert_eq!(replicas::owner(space, doc), None);
        assert!(!replicas::has_doc(space, doc));
        reset();
    }

    #[test]
    fn despawning_peer_cascades_state_and_clears_store() {
        let _g = TEST_LOCK.lock();
        reset();
        let policy = Policy::new();
        let peer = peer(2);
        let space = h(b"cascade-peer-space");
        let doc = h(b"cascade-peer-doc");

        let mut world = World::new();
        world.init_resource::<Policy>();
        let peer_ent = world.spawn(RemotePeer(peer)).id();
        let doc_ent = world.spawn(SpaceDoc { doc, space }).id();
        let pin = PinState::register(policy, peer, doc, space, 1, false).expect("pin");
        world.spawn((pin, StateDoc(doc_ent), StatePeer(peer_ent)));
        assert_eq!(replicas::owner(space, doc), Some(peer));

        world.despawn(peer_ent);
        assert_eq!(replicas::owner(space, doc), None);
        assert!(!replicas::has_doc(space, doc));
        reset();
    }

    #[test]
    fn authority_guard_broadcasts_claim_and_unclaim() {
        let _g = TEST_LOCK.lock();
        reset();
        let policy = Policy::new();
        let me = peer(1);
        set_self_peer_id(me);
        let space = h(b"auth-guard-space");
        let doc = h(b"auth-guard-doc");

        let (token, rx) = replicas::register_stream();
        let _ = rx.try_recv();

        let claim = AuthorityState::register(&policy, me, doc, space, 5, true)
            .expect("authority registers");
        assert_eq!(replicas::authority(space, doc), Some(me));
        assert!(matches!(rx.try_recv(), Ok(StateMsg::Authority { doc: d, .. }) if d == doc));

        drop(claim);
        assert_eq!(replicas::authority(space, doc), None);
        assert!(matches!(rx.try_recv(), Ok(StateMsg::Unclaim { doc: d }) if d == doc));

        replicas::unregister_stream(token);
        reset();
    }

    #[test]
    fn superseded_stream_release_keeps_state() {
        let _g = TEST_LOCK.lock();
        reset();
        let peer = peer(2);
        let space = h(b"supersede-space");
        let doc = h(b"supersede-doc");

        let mut world = World::new();
        world.init_resource::<Policy>();
        let e0 = claim_remote_peer(&mut world, peer, 0);
        assert!(spawn_pin(&mut world, e0, peer, doc, space, 1, false));
        assert_eq!(replicas::owner(space, doc), Some(peer));

        let e1 = claim_remote_peer(&mut world, peer, 1);
        assert_eq!(e0, e1);

        release_remote_peer(&mut world, e0, 0);
        assert_eq!(replicas::owner(space, doc), Some(peer));

        release_remote_peer(&mut world, e1, 1);
        assert_eq!(replicas::owner(space, doc), None);
        reset();
    }

    #[test]
    fn state_tracked_for_unentered_space() {
        let _g = TEST_LOCK.lock();
        reset();
        let peer = peer(2);
        let space = h(b"unentered-space");
        let doc = h(b"unentered-doc");

        // No `Space` entity exists yet.
        let mut world = World::new();
        world.init_resource::<Policy>();
        let peer_ent = world.spawn(RemotePeer(peer)).id();
        assert!(spawn_pin(&mut world, peer_ent, peer, doc, space, 1, false));
        assert_eq!(replicas::owner(space, doc), Some(peer));

        let tracker = entity_by::<SpaceDoc, _>(&mut world, |d| d.doc == doc).expect("tracker");
        assert!(world.get::<ChildOf>(tracker).is_none());
        assert_eq!(world.get::<SpaceDoc>(tracker).map(|d| d.space), Some(space));
        reset();
    }
}
