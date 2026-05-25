//! Root-level document sync.

use loro::LoroDoc;

use crate::{
    error::{
        HydrateError,
        ReconcileError,
    },
    hydrate::Hydrate,
    reconcile::{
        Reconcile,
        RootReconciler,
    },
};

pub trait DocSync: Hydrate + Reconcile {
    const ROOT_KEY: &'static str;

    fn from_doc(doc: &LoroDoc) -> Result<Self, HydrateError> {
        let map = doc.get_map(Self::ROOT_KEY);
        Self::hydrate_map(&map)
    }

    fn to_doc(&self, doc: &LoroDoc) -> Result<(), ReconcileError> {
        let map = doc.get_map(Self::ROOT_KEY);
        let reconciler = RootReconciler::new(map);
        self.reconcile(reconciler)
    }
}
