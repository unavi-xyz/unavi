use lorosurgeon::{Hydrate, HydrateError, NoKey, Reconcile, ReconcileError, Reconciler};
use serde::{Deserialize, Serialize};

use crate::attributes::Attribute;

/// The `name` attribute stores a plain string inline in the attributes map
/// (not a nested struct/map container).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NameAttr(pub String);

impl Hydrate for NameAttr {
    fn hydrate_string(s: &str) -> Result<Self, HydrateError> {
        Ok(Self(s.to_string()))
    }
}

impl Reconcile for NameAttr {
    type Key = NoKey;

    fn reconcile<R: Reconciler>(&self, reconciler: R) -> Result<(), ReconcileError> {
        reconciler.str(&self.0)
    }
}

impl Attribute for NameAttr {
    const KEY: &str = "name";
}
