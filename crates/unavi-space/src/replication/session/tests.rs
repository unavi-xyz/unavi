use hsd::{
    attributes::{
        portal::{
            LinkId,
            PortalAttr,
            PortalDestination,
        },
        script::ScriptAttr,
    },
    id::PrimId,
    property::{
        Payload,
        Property,
        name::PropName,
        value::Value,
    },
    state::stack::MaskKey,
};
use unavi_policy::{
    rules::Rules,
    trust::Trust,
};

use super::*;

fn peer(seed: u8) -> EndpointId {
    iroh::SecretKey::from_bytes(&[seed; 32]).public()
}

fn attribute(payload: Vec<u8>) -> Value {
    Value::Attribute(payload.into())
}

fn portal(link: Option<LinkId>, side: f64) -> Value {
    attribute(
        PortalAttr {
            destination: Some(PortalDestination {
                space: [3; 32],
                link,
            }),
            size_x:      side,
            size_y:      side,
        }
        .encode()
        .expect("encode"),
    )
}

fn frame(side: f64) -> Value {
    attribute(
        PortalAttr {
            destination: None,
            size_x:      side,
            size_y:      side,
        }
        .encode()
        .expect("encode"),
    )
}

fn admits(mask: &Mask, name: &PropName, value: Option<&Value>) -> bool {
    pairs(mask, name, value, None)
}

fn pairs(mask: &Mask, name: &PropName, value: Option<&Value>, beneath: Option<&Value>) -> bool {
    mask.admits(
        PrimId([1; 16]),
        MaskKey::Property {
            name,
            value,
            beneath,
        },
    )
}

#[test]
fn the_authors_layer_contributes_everything() {
    let mask = mask(true, Rules::default().permissions(Trust::Guest));
    assert!(admits(&mask, &ScriptAttr::NAME, Some(&attribute(vec![1]))));
    assert!(mask.admits(PrimId([1; 16]), MaskKey::Parent(None)));
}

#[test]
fn a_peer_granted_nothing_contributes_nothing_even_as_author() {
    let mask = mask(true, Permissions::none());
    assert!(!admits(&mask, &ScriptAttr::NAME, Some(&attribute(vec![1]))));
}

#[test]
fn a_guest_layer_contributes_script_state_and_nothing_the_engine_applies() {
    let mask = mask(false, Rules::default().permissions(Trust::Guest));
    let state: PropName = "state/ticks".parse().expect("name");
    assert!(admits(&mask, &state, Some(&attribute(vec![1]))));
    assert!(admits(&mask, &state, None), "script state may be blocked");
    let bare: PropName = "state".parse().expect("name");
    assert!(
        !admits(&mask, &bare, Some(&attribute(vec![1]))),
        "the group alone is not script state"
    );
    assert!(
        !admits(&mask, &ScriptAttr::NAME, Some(&attribute(vec![1]))),
        "a guest must not decide what the author's document runs"
    );
    assert!(
        !mask.admits(PrimId([1; 16]), MaskKey::Parent(None)),
        "a guest must not remove the author's prims"
    );
}

#[test]
fn a_guest_layer_pairs_only_an_empty_frame_at_its_own_size() {
    let mask = mask(false, Rules::default().permissions(Trust::Guest));
    let link = Some(LinkId([5; 16]));
    let paired = portal(link, 2.0);
    assert!(pairs(
        &mask,
        &PortalAttr::NAME,
        Some(&paired),
        Some(&frame(2.0))
    ));
    assert!(
        !pairs(&mask, &PortalAttr::NAME, None, Some(&frame(2.0))),
        "never a blank"
    );
    assert!(
        !pairs(
            &mask,
            &PortalAttr::NAME,
            Some(&portal(None, 2.0)),
            Some(&frame(2.0))
        ),
        "an unlinked destination is a redirect"
    );
    assert!(
        !pairs(
            &mask,
            &PortalAttr::NAME,
            Some(&portal(link, 3.0)),
            Some(&frame(2.0))
        ),
        "a pairing does not resize the frame"
    );
    assert!(
        !pairs(
            &mask,
            &PortalAttr::NAME,
            Some(&paired),
            Some(&portal(link, 2.0))
        ),
        "a portal that already leads somewhere is not redirected"
    );
    assert!(
        !pairs(&mask, &PortalAttr::NAME, Some(&paired), None),
        "a guest does not add a portal where the author made none"
    );
}

#[test]
fn a_rung_that_opens_no_portals_cannot_pair_one() {
    let mask = mask(false, Rules::default().permissions(Trust::Anonymous));
    assert!(!pairs(
        &mask,
        &PortalAttr::NAME,
        Some(&portal(Some(LinkId([5; 16])), 2.0)),
        Some(&frame(2.0))
    ));
}

#[test]
fn a_blocked_layer_contributes_nothing() {
    let mask = mask(false, Rules::default().permissions(Trust::Blocked));
    let state: PropName = "state/ticks".parse().expect("name");
    assert!(!admits(&mask, &state, Some(&attribute(vec![1]))));
}

#[test]
fn the_authors_devices_stack_strongest_and_others_by_rung() {
    let (author_a, author_b, trusted, guest) = (peer(4), peer(3), peer(1), peer(2));
    let mut layers = vec![
        Standing {
            peer:        author_a,
            is_author:   true,
            trust:       Trust::Guest,
            permissions: Permissions::none(),
        },
        Standing {
            peer:        trusted,
            is_author:   false,
            trust:       Trust::Trusted,
            permissions: Permissions::none(),
        },
        Standing {
            peer:        guest,
            is_author:   false,
            trust:       Trust::Guest,
            permissions: Permissions::none(),
        },
        Standing {
            peer:        author_b,
            is_author:   true,
            trust:       Trust::Guest,
            permissions: Permissions::none(),
        },
    ];
    order(&mut layers);
    let peers = layers.iter().map(|layer| layer.peer).collect::<Vec<_>>();
    let mut authors = [author_a, author_b];
    authors.sort_unstable();
    assert_eq!(
        peers,
        vec![guest, trusted, authors[0], authors[1]],
        "weakest first: guest, then trusted, then the author's devices by id"
    );
}
