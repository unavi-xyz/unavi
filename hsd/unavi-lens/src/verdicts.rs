use std::{
    collections::{
        HashMap,
        HashSet,
    },
    hash::Hash,
};

/// Ticks a refusal stands before the sender is asked about again, about three
/// seconds of fixed updates. A document may be refused only because it was not
/// yet known to be the user's.
const RETRY_TICKS: u64 = 192;

/// Refusals remembered at once; past this the lapsed ones are forgotten.
const REFUSED_MAX: usize = 256;

/// Whether each sender of a message may be acted on.
///
/// The check itself is a host call, so a yes is paid once per sender. A no is
/// held for a few seconds and then asked again, since it can turn into a yes.
#[derive(Debug)]
pub struct Verdicts<K> {
    trusted: HashSet<K>,
    /// Sender to the tick its refusal lapses.
    refused: HashMap<K, u64>,
    now:     u64,
}

impl<K> Default for Verdicts<K> {
    fn default() -> Self {
        Self {
            trusted: HashSet::new(),
            refused: HashMap::new(),
            now:     0,
        }
    }
}

impl<K: Hash + Eq> Verdicts<K> {
    /// Moves the clock on by one fixed update.
    pub const fn tick(&mut self) {
        self.now += 1;
    }

    /// Whether `sender` may be acted on. A host sender, `None`, never is.
    /// `check` answers `None` while it cannot yet tell.
    pub fn trusts(&mut self, sender: Option<K>, check: impl FnOnce(&K) -> Option<bool>) -> bool {
        let Some(sender) = sender else {
            return false;
        };
        if self.trusted.contains(&sender) {
            return true;
        }
        if self
            .refused
            .get(&sender)
            .is_some_and(|&until| until > self.now)
        {
            return false;
        }
        match check(&sender) {
            Some(true) => {
                self.refused.remove(&sender);
                self.trusted.insert(sender);
                true
            }
            Some(false) => {
                if self.refused.len() >= REFUSED_MAX {
                    let now = self.now;
                    self.refused.retain(|_, until| *until > now);
                }
                self.refused.insert(sender, self.now + RETRY_TICKS);
                false
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_yes_is_asked_for_once() {
        let mut verdicts = Verdicts::default();
        let mut asked = 0;
        for _ in 0..3 {
            verdicts.tick();
            assert!(verdicts.trusts(Some(7_u8), |_| {
                asked += 1;
                Some(true)
            }));
        }
        assert_eq!(asked, 1);
    }

    #[test]
    fn a_refusal_stands_for_a_while_then_is_asked_again() {
        let mut verdicts = Verdicts::default();
        assert!(!verdicts.trusts(Some(1_u8), |_| Some(false)));
        verdicts.tick();
        assert!(!verdicts.trusts(Some(1_u8), |_| unreachable!("asked again too soon")));
        for _ in 0..RETRY_TICKS {
            verdicts.tick();
        }
        assert!(verdicts.trusts(Some(1_u8), |_| Some(true)));
    }

    #[test]
    fn an_undecided_check_is_asked_again() {
        let mut verdicts = Verdicts::default();
        assert!(!verdicts.trusts(Some(1_u8), |_| None));
        assert!(verdicts.trusts(Some(1_u8), |_| Some(true)));
    }

    #[test]
    fn the_host_is_never_trusted() {
        let mut verdicts = Verdicts::<u8>::default();
        assert!(!verdicts.trusts(None, |_| unreachable!("host has no document")));
    }

    #[test]
    fn refusals_are_bounded() {
        let mut verdicts = Verdicts::default();
        for sender in 0..=u16::try_from(REFUSED_MAX * 2).expect("small") {
            verdicts.tick();
            verdicts.trusts(Some(sender), |_| Some(false));
        }
        assert!(verdicts.refused.len() <= REFUSED_MAX + 1);
    }
}
