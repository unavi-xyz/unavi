//! Every well-known namespace a node derives from its identity.
//!
//! One list, because a label *is* the document's name: two callers passing the
//! same label to [`crate::identity::RootIdentity::namespace`] address the same
//! document, and a label that changes abandons whatever was written under it.

/// A DID's own document, holding the entries only its owner writes.
pub const ROOT_DOC: &str = "root-doc/v1";

/// A registry's submission catalog.
pub const REGISTRY_CATALOG: &str = "registry-catalog/v1";

pub const REGISTRY_VIEW_RECENT: &str = "registry-view-recent/v1";
pub const REGISTRY_VIEW_FEATURED: &str = "registry-view-featured/v1";
pub const REGISTRY_VIEW_CATEGORIES: &str = "registry-view-categories/v1";
pub const REGISTRY_VIEW_ACTIVE: &str = "registry-view-active/v1";
