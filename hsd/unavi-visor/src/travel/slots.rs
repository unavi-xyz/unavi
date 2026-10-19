//! Which placard stands over which portal.

/// What a pass over the slots changed.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Changes {
    /// Slots whose portal is no longer attended.
    pub released: Vec<usize>,
    /// Slots given a portal they did not have.
    pub claimed:  Vec<usize>,
}

/// Hands the attended keys to the slots. A key that already has a slot keeps
/// it, one that has gone loses it, and a new one takes the first free slot.
/// Attended keys with no slot to take are left out.
pub fn assign<K: Copy + PartialEq>(slots: &mut [Option<K>], attended: &[K]) -> Changes {
    let mut changes = Changes::default();
    for (index, slot) in slots.iter_mut().enumerate() {
        if slot.is_some_and(|key| !attended.contains(&key)) {
            *slot = None;
            changes.released.push(index);
        }
    }
    for &key in attended {
        if slots.contains(&Some(key)) {
            continue;
        }
        if let Some(index) = slots.iter().position(Option::is_none) {
            slots[index] = Some(key);
            changes.claimed.push(index);
        }
    }
    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_keys_take_the_first_free_slots() {
        let mut slots = [None, None, None];
        let changes = assign(&mut slots, &[7, 8]);
        assert_eq!(slots, [Some(7), Some(8), None]);
        assert_eq!(changes.claimed, [0, 1]);
        assert!(changes.released.is_empty());
    }

    #[test]
    fn a_held_key_keeps_its_slot_whatever_the_order() {
        let mut slots = [Some(7), Some(8), None];
        let changes = assign(&mut slots, &[8, 7]);
        assert_eq!(slots, [Some(7), Some(8), None]);
        assert_eq!(changes, Changes::default());
    }

    #[test]
    fn a_gone_key_is_released_and_its_slot_reused() {
        let mut slots = [Some(7), Some(8), None];
        let changes = assign(&mut slots, &[8, 9]);
        assert_eq!(slots, [Some(9), Some(8), None]);
        assert_eq!(changes.released, [0]);
        assert_eq!(changes.claimed, [0]);
    }

    #[test]
    fn nothing_attended_releases_every_slot() {
        let mut slots = [Some(7), None, Some(9)];
        let changes = assign(&mut slots, &[]);
        assert_eq!(slots, [None, None, None]);
        assert_eq!(changes.released, [0, 2]);
        assert!(changes.claimed.is_empty());
    }

    #[test]
    fn more_keys_than_slots_leaves_the_extra_out() {
        let mut slots = [None, None];
        assign(&mut slots, &[1, 2, 3]);
        assert_eq!(slots, [Some(1), Some(2)]);
    }
}
