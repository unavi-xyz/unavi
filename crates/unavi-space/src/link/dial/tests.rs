use std::time::Duration;

use super::{
    MAX_BACKOFF,
    MIN_BACKOFF,
    PeerDial,
};

#[test]
fn a_failing_peer_is_still_due_at_the_capped_interval() {
    let mut dial = PeerDial::default();
    let mut now = Duration::ZERO;

    for _ in 0..1000 {
        assert!(dial.due(now));
        dial.record_failure(now);

        assert!(!dial.due(now));
        let wait = dial.next.saturating_sub(now);
        assert!((MIN_BACKOFF..=MAX_BACKOFF).contains(&wait));
        now = dial.next;
    }

    dial.record_failure(now);
    assert_eq!(dial.next.saturating_sub(now), MAX_BACKOFF);
    assert!(!dial.due((now + MAX_BACKOFF).saturating_sub(Duration::from_secs(1))));
    assert!(dial.due(now + MAX_BACKOFF));
}

#[test]
fn backoff_grows_until_the_cap() {
    let mut dial = PeerDial::default();
    let mut last = Duration::ZERO;
    for _ in 0..10 {
        dial.record_failure(Duration::ZERO);
        assert!(dial.next >= last);
        last = dial.next;
    }
    assert_eq!(last, MAX_BACKOFF);
}
