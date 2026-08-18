use hsd::id::DocId;
use iroh_docs::NamespaceId;

use crate::state::replicas;

/// The space a document belongs to.
///
/// Either the space it was registered into, or — for a pinned document, which
/// is namespace-backed and has no local registration — the space some peer's
/// pin names.
#[must_use]
pub fn doc_space(doc: DocId) -> Option<DocId> {
    unavi_policy::membership::registered_space(doc)
        .or_else(|| replicas::space_of(NamespaceId::from(&doc.0)).map(|ns| DocId(*ns.as_bytes())))
}

#[must_use]
pub fn same_space(a: DocId, b: DocId) -> bool {
    match (doc_space(a), doc_space(b)) {
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}
