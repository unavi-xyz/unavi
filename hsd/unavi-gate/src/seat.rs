//! Which beacon sits in the socket, decided from where each one is.

use wired_guest::beacon::DocumentId;

/// Metres from the seat point within which a beacon takes an empty socket.
pub const SEAT_DISTANCE: f32 = 0.04;
/// Metres from the seat point a seated beacon must leave to be unseated.
pub const UNSEAT_DISTANCE: f32 = 0.09;

/// A beacon heard near the socket, measured this tick.
#[derive(Clone, Copy, Debug)]
pub struct Candidate {
    pub beacon:   DocumentId,
    pub space:    DocumentId,
    /// Metres from its body to the seat point.
    pub distance: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Occupant {
    pub beacon: DocumentId,
    pub space:  DocumentId,
}

#[derive(Default)]
pub struct Seat {
    occupant: Option<Occupant>,
}

impl Seat {
    /// Keeps the occupant while it stays within [`UNSEAT_DISTANCE`], else
    /// seats the nearest candidate within [`SEAT_DISTANCE`].
    pub fn update(&mut self, candidates: &[Candidate]) -> Option<Occupant> {
        let kept = self.occupant.and_then(|occupant| {
            candidates
                .iter()
                .find(|c| c.beacon == occupant.beacon && c.distance < UNSEAT_DISTANCE)
                .map(|c| Occupant {
                    beacon: c.beacon,
                    space:  c.space,
                })
        });
        self.occupant = kept.or_else(|| {
            candidates
                .iter()
                .filter(|c| c.distance < SEAT_DISTANCE)
                .min_by(|a, b| a.distance.total_cmp(&b.distance))
                .map(|c| Occupant {
                    beacon: c.beacon,
                    space:  c.space,
                })
        });
        self.occupant
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: DocumentId = (1, 0, 0, 0);
    const B: DocumentId = (2, 0, 0, 0);
    const SPACE_A: DocumentId = (10, 0, 0, 0);
    const SPACE_B: DocumentId = (20, 0, 0, 0);

    fn at(beacon: DocumentId, space: DocumentId, distance: f32) -> Candidate {
        Candidate {
            beacon,
            space,
            distance,
        }
    }

    #[test]
    fn a_seated_beacon_holds_the_socket_against_a_nearer_one() {
        let mut seat = Seat::default();
        seat.update(&[at(A, SPACE_A, 0.01)]);
        let occupant = seat.update(&[at(A, SPACE_A, 0.06), at(B, SPACE_B, 0.0)]);
        assert_eq!(
            occupant.map(|o| o.beacon),
            Some(A),
            "a beacon pushed toward a full socket must not swap the destination"
        );
    }

    #[test]
    fn a_beacon_is_unseated_only_past_the_wider_distance() {
        let mut seat = Seat::default();
        seat.update(&[at(A, SPACE_A, 0.01)]);
        assert!(seat.update(&[at(A, SPACE_A, 0.08)]).is_some());
        assert!(seat.update(&[at(A, SPACE_A, 0.10)]).is_none());
        assert!(
            seat.update(&[at(A, SPACE_A, 0.08)]).is_none(),
            "between the two distances an empty socket stays empty"
        );
    }

    #[test]
    fn a_beacon_no_longer_heard_leaves_the_socket() {
        let mut seat = Seat::default();
        seat.update(&[at(A, SPACE_A, 0.01)]);
        assert!(seat.update(&[]).is_none());
    }
}
