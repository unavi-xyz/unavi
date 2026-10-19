use std::{
    collections::BTreeMap,
    sync::Arc,
};

use hsd::{
    attributes::portal::LinkId,
    id::DocId,
};

use super::{
    IntentReceiver,
    LinkedPortal,
    derive_intents,
};
use crate::host::{
    queue::{
        DEPTH,
        Queue,
    },
    shared_state::link_intents::{
        LinkIntent,
        LinkIntents,
        Offer,
    },
};

const SPACE_A: DocId = DocId([1; 32]);
const SPACE_B: DocId = DocId([2; 32]);
const GATE_A: DocId = DocId([10; 32]);
const GATE_B: DocId = DocId([20; 32]);
const OTHER_B: DocId = DocId([21; 32]);
const LINK: LinkId = LinkId([9; 16]);

const NEAR: LinkedPortal = LinkedPortal {
    home:   SPACE_A,
    target: SPACE_B,
    link:   LINK,
};
const FAR: LinkedPortal = LinkedPortal {
    home:   SPACE_B,
    target: SPACE_A,
    link:   LINK,
};

const RECEIVERS: [IntentReceiver; 3] = [
    IntentReceiver {
        doc:   GATE_A,
        space: SPACE_A,
    },
    IntentReceiver {
        doc:   GATE_B,
        space: SPACE_B,
    },
    IntentReceiver {
        doc:   OTHER_B,
        space: SPACE_B,
    },
];

#[test]
fn an_unanswered_link_reaches_every_document_in_its_target() {
    let intents = derive_intents(&[NEAR], &RECEIVERS);

    let intent = LinkIntent {
        source_space: SPACE_A,
        link:         LINK,
    };
    assert_eq!(intents.len(), 1);
    assert_eq!(intents[&intent], vec![GATE_B, OTHER_B]);
}

#[test]
fn an_answered_link_raises_nothing() {
    assert!(derive_intents(&[NEAR, FAR], &RECEIVERS).is_empty());
}

#[test]
fn an_unloaded_target_raises_nothing() {
    assert!(derive_intents(&[NEAR], &RECEIVERS[..1]).is_empty());
}

fn raised() -> BTreeMap<LinkIntent, Vec<DocId>> {
    derive_intents(&[NEAR], &RECEIVERS)
}

fn subscribe(intents: &LinkIntents, doc: DocId) -> (u64, Arc<Queue<Offer>>) {
    let queue = Arc::new(Queue::default());
    (intents.open(doc, Arc::clone(&queue)), queue)
}

#[test]
fn a_late_subscriber_receives_a_still_unclaimed_intent() {
    let intents = LinkIntents::default();
    let (_, first) = subscribe(&intents, GATE_B);
    assert_eq!(intents.reconcile(&raised()), 1);
    assert_eq!(first.drain(8).len(), 1);

    let (_, late) = subscribe(&intents, OTHER_B);
    assert_eq!(intents.reconcile(&raised()), 1);
    assert_eq!(late.drain(8).len(), 1);
    assert!(first.drain(8).is_empty());
}

#[test]
fn a_resubscribing_script_is_offered_again() {
    let intents = LinkIntents::default();
    let (id, queue) = subscribe(&intents, GATE_B);
    intents.reconcile(&raised());
    assert_eq!(queue.drain(8).len(), 1);

    intents.close(id);
    let (_, queue) = subscribe(&intents, GATE_B);
    intents.reconcile(&raised());
    assert_eq!(queue.drain(8).len(), 1);
}

#[test]
fn an_offer_is_not_repeated_to_the_same_subscription() {
    let intents = LinkIntents::default();
    let (_, queue) = subscribe(&intents, GATE_B);
    intents.reconcile(&raised());
    assert_eq!(intents.reconcile(&raised()), 0);
    assert_eq!(queue.drain(8).len(), 1);
}

#[test]
fn a_queue_overflow_is_offered_again() {
    let intents = LinkIntents::default();
    let (_, queue) = subscribe(&intents, GATE_B);
    let many = (0..=DEPTH as u8)
        .map(|i| {
            let intent = LinkIntent {
                source_space: SPACE_A,
                link:         LinkId([i; 16]),
            };
            (intent, vec![GATE_B])
        })
        .collect::<BTreeMap<_, _>>();

    assert_eq!(intents.reconcile(&many), DEPTH + 1);
    assert_eq!(queue.dropped(), 1);
    assert_eq!(queue.drain(DEPTH as u32).len(), DEPTH);

    assert_eq!(intents.reconcile(&many), DEPTH + 1);
}

#[test]
fn a_claimed_intent_is_not_offered_again() {
    let intents = LinkIntents::default();
    let (_, winner) = subscribe(&intents, GATE_B);
    intents.reconcile(&raised());
    assert!(winner.drain(8)[0].take());

    let (_, late) = subscribe(&intents, OTHER_B);
    assert_eq!(intents.reconcile(&raised()), 0);
    assert!(late.drain(8).is_empty());
}

#[test]
fn one_emission_is_claimed_by_a_single_subscriber() {
    let intents = LinkIntents::default();
    let (_, a) = subscribe(&intents, GATE_B);
    let (_, b) = subscribe(&intents, OTHER_B);
    intents.reconcile(&raised());

    let offer_a = a.drain(8).remove(0);
    let offer_b = b.drain(8).remove(0);
    assert!(offer_a.take());
    assert!(!offer_b.take());
    assert!(!offer_a.take());
}

#[test]
fn an_intent_claimed_by_a_dead_script_is_offered_again() {
    let intents = LinkIntents::default();
    let (dead, queue) = subscribe(&intents, GATE_B);
    let (_, other) = subscribe(&intents, OTHER_B);
    intents.reconcile(&raised());
    let stale = other.drain(8).remove(0);
    assert!(queue.drain(8)[0].take());
    assert_eq!(intents.reconcile(&raised()), 0);

    intents.close(dead);
    assert_eq!(intents.reconcile(&raised()), 1);
    let fresh = other.drain(8).remove(0);
    assert!(!stale.take());
    assert!(fresh.take());
}

#[test]
fn a_withdrawn_intent_is_asked_for_again_when_it_returns() {
    let intents = LinkIntents::default();
    let (_, queue) = subscribe(&intents, GATE_B);
    intents.reconcile(&raised());
    assert!(queue.drain(8)[0].take());

    intents.reconcile(&BTreeMap::new());
    intents.reconcile(&raised());
    assert!(queue.drain(8)[0].take());
}
