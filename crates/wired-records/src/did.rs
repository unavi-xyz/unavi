use loro::ValueOrContainer;
use lorosurgeon::{Hydrate, Reconcile, Reconciler};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use xdid::core::did::Did;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HydratedDid(pub Did);

impl Hydrate for HydratedDid {
    fn hydrate(source: &ValueOrContainer) -> Result<Self, lorosurgeon::HydrateError> {
        let string_val = <String as Hydrate>::hydrate(source)?;
        Did::from_str(&string_val)
            .map(HydratedDid)
            .map_err(|_| lorosurgeon::HydrateError::unexpected("valid Did", "invalid Did"))
    }
}

impl Reconcile for HydratedDid {
    type Key = lorosurgeon::NoKey;

    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), lorosurgeon::ReconcileError> {
        let string_val = self.0.to_string();
        string_val.reconcile(r)
    }
}
