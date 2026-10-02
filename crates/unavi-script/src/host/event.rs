//! `wired:event`: messages between scripts on this peer.

use std::{
    collections::{
        HashSet,
        VecDeque,
    },
    sync::{
        Arc,
        atomic::AtomicBool,
    },
};

use hsd::{
    bounds::MAX_EVENT_PAYLOAD_BYTES,
    id::{
        DocId,
        PrimId,
    },
};
use unavi_physics::finite;
use unavi_policy::{
    permissions::HostApi,
    quota::{
        Flow,
        Stock,
        StockLease,
    },
};

use crate::{
    error::ScriptError,
    host::{
        ScriptHost,
        queue::Queue,
        scene::node,
        shared_state::event_bus::{
            self,
            Delivery,
            Emit,
            EventBus,
            Listener,
            Scope as BusScope,
        },
    },
};

/// Longest channel name.
pub const MAX_CHANNEL_BYTES: usize = 256;
/// Most channels one listener hears.
pub const MAX_CHANNELS: usize = 32;
/// Most documents one `to` or `from` filter names.
pub const MAX_FILTER_DOCUMENTS: usize = 64;
/// Claims a subscription remembers. Older tokens can no longer be claimed.
const MAX_CLAIMS: usize = 256;

#[derive(Clone, Copy, Debug)]
pub enum Scope {
    Global,
    Spatial {
        doc:    DocId,
        prim:   PrimId,
        radius: f32,
    },
}

/// One message as a script reads it.
pub struct Message {
    pub channel:     String,
    pub payload:     Vec<u8>,
    pub sender:      DocId,
    pub distance:    Option<f32>,
    pub sent_at:     f64,
    pub claim_token: Option<u64>,
}

/// A guest's open message subscription. Closes its listener when dropped.
pub struct MessageSubscription {
    id:         u64,
    bus:        EventBus,
    queue:      Arc<Queue<Delivery>>,
    /// Claims of addressed messages drained so far, newest last.
    claims:     VecDeque<(u64, Arc<AtomicBool>)>,
    next_token: u64,
    _lease:     StockLease,
}

impl Drop for MessageSubscription {
    fn drop(&mut self) {
        self.bus.close(self.id);
    }
}

fn check_channel(channel: &str) -> Result<(), ScriptError> {
    if channel.len() > MAX_CHANNEL_BYTES {
        return Err(ScriptError::invalid("a channel is at most 256 bytes"));
    }
    Ok(())
}

fn filter(docs: Option<Vec<DocId>>) -> Result<Option<HashSet<DocId>>, ScriptError> {
    match docs {
        Some(docs) if docs.len() > MAX_FILTER_DOCUMENTS => {
            Err(ScriptError::invalid("a filter names at most 64 documents"))
        }
        docs => Ok(docs.map(|docs| docs.into_iter().collect())),
    }
}

/// A spatial scope must sit on a prim of a document the script may write, so
/// a script cannot speak or listen from somewhere else's position.
fn bus_scope(host: &ScriptHost, scope: Scope) -> Result<BusScope, ScriptError> {
    match scope {
        Scope::Global => Ok(BusScope::Global),
        Scope::Spatial { doc, prim, radius } => {
            if !finite::nonnegative_length(radius) {
                return Err(ScriptError::invalid(
                    "a radius must be finite and not negative",
                ));
            }
            if !host.may_write(doc) {
                return Err(ScriptError::Forbidden);
            }
            Ok(BusScope::Spatial {
                origin: node(doc, prim),
                radius,
            })
        }
    }
}

pub fn emit(
    host: &ScriptHost,
    channel: &str,
    payload: Vec<u8>,
    to: Option<Vec<DocId>>,
    scope: Scope,
) -> Result<(), ScriptError> {
    host.require(HostApi::Event)?;
    check_channel(channel)?;
    if payload.len() > MAX_EVENT_PAYLOAD_BYTES {
        return Err(ScriptError::invalid("a payload is at most 64 KiB"));
    }
    let to = filter(to)?;
    let scope = bus_scope(host, scope)?;
    crate::quota::take(&host.quota, Flow::Emit, 1)?;

    host.shared.event_bus.deliver(
        &Emit {
            channel,
            payload: Arc::from(payload),
            sender: host.doc,
            to: to.as_ref(),
            scope,
            sent_at: host.now,
        },
        &host.shared.transforms,
    );
    Ok(())
}

pub fn listen(
    host: &mut ScriptHost,
    channels: Vec<String>,
    from: Option<Vec<DocId>>,
    scope: Scope,
) -> Result<u32, ScriptError> {
    host.require(HostApi::Event)?;
    if channels.len() > MAX_CHANNELS {
        return Err(ScriptError::invalid("a listener hears at most 32 channels"));
    }
    channels.iter().try_for_each(|c| check_channel(c))?;
    let from = filter(from)?;
    let scope = bus_scope(host, scope)?;
    let lease = host.quota.lease(Stock::Receptors, 1)?;

    let queue = Arc::new(Queue::default());
    let bus = host.shared.event_bus.clone();
    let id = bus.listen(Listener {
        doc: host.doc,
        channels,
        from,
        scope,
        queue: Arc::clone(&queue),
    });
    let subscription = MessageSubscription {
        id,
        bus,
        queue,
        claims: VecDeque::new(),
        next_token: 0,
        _lease: lease,
    };
    Ok(host.messages.insert(subscription, &host.quota)?)
}

pub fn drain(
    host: &mut ScriptHost,
    subscription: u32,
    max: u32,
) -> Result<Vec<Message>, ScriptError> {
    let sub = host.messages.get_mut(subscription)?;
    let deliveries = sub.queue.drain(max);
    Ok(deliveries
        .into_iter()
        .map(|delivery| {
            let claim_token = delivery.claim.map(|claim| {
                let token = sub.next_token;
                sub.next_token += 1;
                if sub.claims.len() >= MAX_CLAIMS {
                    sub.claims.pop_front();
                }
                sub.claims.push_back((token, claim));
                token
            });
            Message {
                channel: delivery.channel.to_string(),
                payload: delivery.payload.to_vec(),
                sender: delivery.sender,
                distance: delivery.distance,
                sent_at: delivery.sent_at,
                claim_token,
            }
        })
        .collect())
}

pub fn dropped(host: &ScriptHost, subscription: u32) -> Result<u64, ScriptError> {
    Ok(host.messages.get(subscription)?.queue.dropped())
}

/// Whether this subscription won the message `token` names. An unknown or
/// expired token loses.
pub fn claim(host: &ScriptHost, subscription: u32, token: u64) -> Result<bool, ScriptError> {
    let sub = host.messages.get(subscription)?;
    Ok(sub
        .claims
        .iter()
        .find(|(held, _)| *held == token)
        .is_some_and(|(_, claim)| event_bus::claim(claim)))
}

pub fn close(host: &mut ScriptHost, subscription: u32) -> Result<(), ScriptError> {
    host.messages.remove(subscription).map(drop)
}
