use loro_surgeon::{
    Hydrate,
    Reconcile,
    bytes::ByteArray,
    error::{
        HydrateError,
        ReconcileError,
    },
    reconcile::{
        NoKey,
        Reconciler,
    },
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::attributes::Attribute;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ScriptAttr(pub ByteArray<32>);

impl Hydrate for ScriptAttr {
    fn hydrate_binary(b: &[u8]) -> Result<Self, HydrateError> {
        Ok(Self(ByteArray::<32>::hydrate_binary(b)?))
    }
}

impl Reconcile for ScriptAttr {
    type Key = NoKey;

    fn reconcile<R: Reconciler>(&self, reconciler: R) -> Result<(), ReconcileError> {
        self.0.reconcile(reconciler)
    }
}

impl Attribute for ScriptAttr {
    const KEY: &str = "script";
}
