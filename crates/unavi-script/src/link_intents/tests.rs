use hsd::{
    attributes::portal::LinkId,
    id::DocId,
};

use super::{
    IntentReceiver,
    LinkedPortal,
    derive_intents,
};
use crate::host::shared_state::link_intents::LinkIntent;

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
