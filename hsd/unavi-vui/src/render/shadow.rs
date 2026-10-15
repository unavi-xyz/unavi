//! One cache for "did this value change since the last write", so a prim
//! property is written when there is something new to say and not every
//! frame regardless.

use std::cell::RefCell;

/// The last value handed to a prim property.
pub struct Shadow<T>(RefCell<Option<T>>);

impl<T> Default for Shadow<T> {
    fn default() -> Self {
        Self(RefCell::new(None))
    }
}

impl<T: Clone + PartialEq> Shadow<T> {
    pub const fn new() -> Self {
        Self(RefCell::new(None))
    }

    /// `value`, once it differs from what this shadow last held; `None` once
    /// a caller has already written it.
    pub fn diff(&self, value: T) -> Option<T> {
        let mut last = self.0.borrow_mut();
        if last.as_ref() == Some(&value) {
            return None;
        }
        *last = Some(value.clone());
        Some(value)
    }

    /// Forgets what was written, so the next value is written regardless of
    /// what it is.
    pub fn clear(&self) {
        *self.0.borrow_mut() = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_value_is_always_written() {
        let shadow = Shadow::new();
        assert_eq!(shadow.diff(1), Some(1));
    }

    #[test]
    fn an_unchanged_value_is_not_written_again() {
        let shadow = Shadow::new();
        shadow.diff(1);
        assert_eq!(shadow.diff(1), None);
    }

    #[test]
    fn a_changed_value_is_written() {
        let shadow = Shadow::new();
        shadow.diff(1);
        assert_eq!(shadow.diff(2), Some(2));
    }

    #[test]
    fn clearing_forces_the_next_value_to_write() {
        let shadow = Shadow::new();
        shadow.diff(1);
        shadow.clear();
        assert_eq!(shadow.diff(1), Some(1));
    }
}
