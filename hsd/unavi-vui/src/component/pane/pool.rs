//! Which prim slots a pane's runs must be written to.
//!
//! A run is keyed by its whole content: a slot already holding an equal run
//! is left alone, so a pane at rest writes nothing and a changed line writes
//! one prim.

/// What to do to the slots this tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan<T> {
    pub writes: Vec<(usize, T)>,
    /// Slots that held a run and no longer do.
    pub hidden: Vec<usize>,
}

/// The run each slot last held.
#[derive(Debug, Clone)]
pub struct Pool<T> {
    slots: Vec<Option<T>>,
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self { slots: Vec::new() }
    }
}

impl<T: Clone + PartialEq> Pool<T> {
    /// Slots touched so far, which is how many prims exist for them.
    #[must_use]
    pub const fn slots(&self) -> usize {
        self.slots.len()
    }

    /// Assigns `wanted` to slots from the first and says what changed.
    pub fn plan(&mut self, wanted: Vec<T>) -> Plan<T> {
        let count = wanted.len();
        let mut writes = Vec::new();
        for (index, run) in wanted.into_iter().enumerate() {
            if index == self.slots.len() {
                self.slots.push(None);
            }
            if self.slots[index].as_ref() != Some(&run) {
                self.slots[index] = Some(run.clone());
                writes.push((index, run));
            }
        }
        let mut hidden = Vec::new();
        for (index, slot) in self.slots.iter_mut().enumerate().skip(count) {
            if slot.take().is_some() {
                hidden.push(index);
            }
        }
        Plan { writes, hidden }
    }

    /// Empties every slot, returning those that held a run.
    pub fn clear(&mut self) -> Vec<usize> {
        self.plan(Vec::new()).hidden
    }
}
