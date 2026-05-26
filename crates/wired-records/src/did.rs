use std::str::FromStr;

use loro::ValueOrContainer;
use loro_surgeon::{
    Hydrate,
    Reconcile,
    reconcile::Reconciler,
};
use serde::{
    Deserialize,
    Serialize,
};
use xdid::core::did::Did;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HydratedDid(pub Did);

impl Hydrate for HydratedDid {
    fn hydrate(source: &ValueOrContainer) -> Result<Self, loro_surgeon::error::HydrateError> {
        let string_val = <String as Hydrate>::hydrate(source)?;
        Did::from_str(&string_val)
            .map(HydratedDid)
            .map_err(|_| loro_surgeon::error::HydrateError::unexpected("valid Did", "invalid Did"))
    }
}

impl Reconcile for HydratedDid {
    type Key = loro_surgeon::reconcile::NoKey;

    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), loro_surgeon::error::ReconcileError> {
        let string_val = self.0.to_string();
        string_val.reconcile(r)
    }
}
