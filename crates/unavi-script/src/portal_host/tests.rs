use hsd::{
    attributes::portal::LinkId,
    id::DocId,
};
use unavi_portal_protocol::LinkIntent;

use super::{
    IntentReceiver,
    LinkedPortal,
    derive_intents,
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
fn test_unanswered_link_reaches_every_target_doc() {
    let intents = derive_intents(&[NEAR], &RECEIVERS);

    let intent = LinkIntent {
        source_space: SPACE_A.0,
        link:         LINK.0,
    };
    assert_eq!(intents.len(), 1);
    assert_eq!(intents[&intent], vec![GATE_B, OTHER_B]);
}

#[test]
fn test_answered_link_derives_nothing() {
    assert!(derive_intents(&[NEAR, FAR], &RECEIVERS).is_empty());
}

#[test]
fn test_unloaded_target_derives_nothing() {
    assert!(derive_intents(&[NEAR], &RECEIVERS[..1]).is_empty());
}
