//! Reconcile `Vec<T>` into a `LoroList` using Myers LCS diffing.

use loro::ValueOrContainer;
use similar::algorithms::DiffHook;

use crate::{
    error::ReconcileError,
    hydrate::Hydrate,
    reconcile::{ListReconciler, PropReconciler, Reconcile, Reconciler},
};

impl ListReconciler {
    #[must_use]
    pub fn get(&self, index: usize) -> Option<ValueOrContainer> {
        self.list.get(index)
    }

    pub fn insert<R: Reconcile>(&mut self, index: usize, value: &R) -> Result<(), ReconcileError> {
        let reconciler = PropReconciler::list_insert(self.list.clone(), index);
        value.reconcile(reconciler)
    }

    pub fn delete(&mut self, index: usize) -> Result<(), ReconcileError> {
        self.list.delete(index, 1)?;
        Ok(())
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }
}

pub fn reconcile_vec<T, R>(items: &[T], r: R) -> Result<(), ReconcileError>
where
    T: Reconcile + Hydrate + PartialEq,
    R: Reconciler,
{
    let mut list_r = r.list()?;
    let old_len = list_r.len();

    if old_len == 0 && items.is_empty() {
        return Ok(());
    }

    if old_len == 0 {
        for (i, item) in items.iter().enumerate() {
            list_r.insert(i, item)?;
        }
        return Ok(());
    }

    let old: Vec<HydratedItem<T>> = (0..old_len)
        .map(|i| HydratedItem(list_r.get(i).and_then(|voc| T::hydrate(&voc).ok())))
        .collect();

    let mut hook = LcsHook {
        idx: 0,
        list: &mut list_r,
        items,
    };

    similar::algorithms::myers::diff(&mut hook, items, 0..items.len(), &old, 0..old.len())?;

    Ok(())
}

struct HydratedItem<T>(Option<T>);

impl<T: PartialEq> PartialEq<T> for HydratedItem<T> {
    fn eq(&self, other: &T) -> bool {
        self.0.as_ref().is_some_and(|v| v == other)
    }
}

struct LcsHook<'a, T> {
    idx: usize,
    list: &'a mut ListReconciler,
    items: &'a [T],
}

impl<T: Reconcile> DiffHook for LcsHook<'_, T> {
    type Error = ReconcileError;

    fn equal(
        &mut self,
        _old_index: usize,
        _new_index: usize,
        len: usize,
    ) -> Result<(), Self::Error> {
        self.idx += len;
        Ok(())
    }

    fn delete(
        &mut self,
        old_index: usize,
        old_len: usize,
        _new_index: usize,
    ) -> Result<(), Self::Error> {
        for i in 0..old_len {
            self.list.insert(self.idx, &self.items[old_index + i])?;
            self.idx += 1;
        }
        Ok(())
    }

    fn insert(
        &mut self,
        _old_index: usize,
        _new_index: usize,
        new_len: usize,
    ) -> Result<(), Self::Error> {
        for _ in 0..new_len {
            self.list.delete(self.idx)?;
        }
        Ok(())
    }
}

pub fn reconcile_vec_simple<T: Reconcile, R: Reconciler>(
    items: &[T],
    r: R,
) -> Result<(), ReconcileError> {
    let mut list_r = r.list()?;

    while !list_r.is_empty() {
        list_r.delete(0)?;
    }

    for (i, item) in items.iter().enumerate() {
        list_r.insert(i, item)?;
    }

    Ok(())
}

pub fn reconcile_vec_movable<T: Reconcile, R: Reconciler>(
    items: &[T],
    r: R,
) -> Result<(), ReconcileError> {
    let mut list_r = r.movable_list()?;
    super::movable_list::reconcile_movable_list(items, &mut list_r)
}
