//! Reconcile `Vec<T>` into a [`LoroMovableList`] with key-based diffing.

use std::collections::HashMap;

use loro::ValueOrContainer;

use crate::{error::ReconcileError, reconcile::{LoadKey, MovableListReconciler, PropReconciler, Reconcile}};

impl MovableListReconciler {
    #[must_use]
    pub fn get(&self, index: usize) -> Option<ValueOrContainer> {
        self.list.get(index)
    }

    pub fn set<R: Reconcile>(&mut self, index: usize, value: &R) -> Result<(), ReconcileError> {
        let reconciler = PropReconciler::movable_list_set(self.list.clone(), index);
        value.reconcile(reconciler)
    }

    pub fn insert<R: Reconcile>(&mut self, index: usize, value: &R) -> Result<(), ReconcileError> {
        let reconciler = PropReconciler::movable_list_insert(self.list.clone(), index);
        value.reconcile(reconciler)
    }

    pub fn delete(&mut self, index: usize) -> Result<(), ReconcileError> {
        self.list.delete(index, 1)?;
        Ok(())
    }

    pub fn mov(&mut self, from: usize, to: usize) -> Result<(), ReconcileError> {
        self.list.mov(from, to)?;
        Ok(())
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.list.len() == 0
    }
}

pub fn reconcile_movable_list<T: Reconcile>(
    items: &[T],
    list_r: &mut MovableListReconciler,
) -> Result<(), ReconcileError> {
    let old_len = list_r.len();

    let has_keys = items
        .first()
        .is_some_and(|item| !matches!(item.key(), LoadKey::NoKey));

    if !has_keys {
        return reconcile_positional(items, list_r, old_len);
    }

    reconcile_keyed(items, list_r, old_len)
}

fn reconcile_positional<T: Reconcile>(
    items: &[T],
    list_r: &mut MovableListReconciler,
    old_len: usize,
) -> Result<(), ReconcileError> {
    let overlap = old_len.min(items.len());
    for (i, item) in items[..overlap].iter().enumerate() {
        list_r.set(i, item)?;
    }

    for _ in items.len()..old_len {
        list_r.delete(items.len())?;
    }

    if items.len() > old_len {
        for (i, item) in items[old_len..].iter().enumerate() {
            list_r.insert(old_len + i, item)?;
        }
    }

    Ok(())
}

fn reconcile_keyed<T: Reconcile>(
    items: &[T],
    list_r: &mut MovableListReconciler,
    old_len: usize,
) -> Result<(), ReconcileError> {
    let old_keys: Vec<Option<T::Key>> = (0..old_len)
        .map(|i| {
            list_r
                .get(i)
                .and_then(|voc| T::hydrate_key(&voc).ok())
                .and_then(LoadKey::into_found)
        })
        .collect();

    let mut key_to_old: HashMap<&T::Key, Vec<usize>> = HashMap::with_capacity(old_len);
    for (i, key) in old_keys.iter().enumerate() {
        if let Some(k) = key {
            key_to_old.entry(k).or_default().push(i);
        }
    }

    let mut old_used = vec![false; old_len];
    let mut new_to_old: Vec<Option<usize>> = Vec::with_capacity(items.len());

    for item in items {
        let matched = item.key().into_found().and_then(|nk| {
            key_to_old.get_mut(&nk).and_then(|indices| {
                indices
                    .iter()
                    .position(|&idx| !old_used[idx])
                    .map(|pos| indices[pos])
            })
        });

        if let Some(old_idx) = matched {
            old_used[old_idx] = true;
            new_to_old.push(Some(old_idx));
        } else {
            new_to_old.push(None);
        }
    }

    for idx in (0..old_len).rev() {
        if !old_used[idx] {
            list_r.delete(idx)?;
        }
    }

    let mut current_order: Vec<usize> = (0..old_len).filter(|i| old_used[*i]).collect();

    for (target_idx, maybe_old) in new_to_old.iter().enumerate() {
        if let Some(old_idx) = maybe_old {
            let current_pos = current_order
                .iter()
                .position(|&x| x == *old_idx)
                .expect("matched item must exist in current_order");

            if current_pos != target_idx {
                list_r.mov(current_pos, target_idx)?;
                let val = current_order.remove(current_pos);
                current_order.insert(target_idx, val);
            }
            list_r.set(target_idx, &items[target_idx])?;
        } else {
            list_r.insert(target_idx, &items[target_idx])?;
            current_order.insert(target_idx, usize::MAX);
        }
    }

    Ok(())
}
