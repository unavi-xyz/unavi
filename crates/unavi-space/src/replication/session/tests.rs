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

fn admits(mask: &Mask, name: &PropName, value: Option<&Value>) -> bool {
    mask.admits(PrimId([1; 16]), MaskKey::Property(name, value))
}

#[test]
fn the_authors_layer_contributes_everything() {
    let mask = mask(true, Permissions::none());
    assert!(admits(&mask, &ScriptAttr::NAME, Some(&attribute(vec![1]))));
    assert!(mask.admits(PrimId([1; 16]), MaskKey::Parent(None)));
}

#[test]
fn a_guest_layer_contributes_script_state_and_nothing_the_engine_applies() {
    let mask = mask(false, Rules::default().permissions(Trust::Guest));
    let state: PropName = "state/ticks".parse().expect("name");
    assert!(admits(&mask, &state, Some(&attribute(vec![1]))));
    assert!(admits(&mask, &state, None), "script state may be blocked");
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
fn a_guest_layer_contributes_a_portal_only_as_a_pairing() {
    let mask = mask(false, Rules::default().permissions(Trust::Guest));
    let link = Some(LinkId([5; 16]));
    assert!(admits(&mask, &PortalAttr::NAME, Some(&portal(link, 2.0))));
    assert!(!admits(&mask, &PortalAttr::NAME, None), "never a blank");
    assert!(
        !admits(&mask, &PortalAttr::NAME, Some(&portal(None, 2.0))),
        "an unlinked destination is a redirect"
    );
    assert!(!admits(
        &mask,
        &PortalAttr::NAME,
        Some(&portal(link, f64::NAN))
    ));
    assert!(!admits(
        &mask,
        &PortalAttr::NAME,
        Some(&portal(link, 1_000.0))
    ));
}

#[test]
fn a_rung_that_opens_no_portals_cannot_pair_one() {
    let mask = mask(false, Rules::default().permissions(Trust::Anonymous));
    assert!(!admits(
        &mask,
        &PortalAttr::NAME,
        Some(&portal(Some(LinkId([5; 16])), 2.0))
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
            peer:      author_a,
            is_author: true,
            trust:     Trust::Guest,
        },
        Standing {
            peer:      trusted,
            is_author: false,
            trust:     Trust::Trusted,
        },
        Standing {
            peer:      guest,
            is_author: false,
            trust:     Trust::Guest,
        },
        Standing {
            peer:      author_b,
            is_author: true,
            trust:     Trust::Guest,
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
