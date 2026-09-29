use hsd::id::{
    DocId,
    PrimId,
};

use super::derive_link;

const DOC: DocId = DocId([1; 32]);
const PRIM: PrimId = PrimId([2; 16]);

#[test]
fn test_link_is_stable_per_portal_and_target() {
    assert_eq!(
        derive_link(DOC, PRIM, [3; 32]),
        derive_link(DOC, PRIM, [3; 32])
    );
    assert_ne!(
        derive_link(DOC, PRIM, [3; 32]),
        derive_link(DOC, PRIM, [4; 32])
    );
    assert_ne!(
        derive_link(DOC, PRIM, [3; 32]),
        derive_link(DOC, PrimId([5; 16]), [3; 32])
    );
}
