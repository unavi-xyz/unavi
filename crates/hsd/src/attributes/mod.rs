use loro::LoroMap;
use lorosurgeon::{Hydrate, HydrateError, Reconcile, ReconcileError, reconcile::PropReconciler};

pub mod xform;

pub trait Attribute: Reconcile + Hydrate {
    const KEY: &str;

    fn attr_hydrate(map: &LoroMap) -> Result<Self, HydrateError> {
        lorosurgeon::hydrate_prop(map, Self::KEY)
    }

    fn attr_reconcile(&self, map: LoroMap) -> Result<(), ReconcileError> {
        let rec = PropReconciler::map_put(map, Self::KEY.to_string());
        self.reconcile(rec)
    }
}
