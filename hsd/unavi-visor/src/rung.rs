//! How a trust rung reads on a placard.

use crate::wired::perception::perception::TrustRung;

/// The rung's word, or `None` for the local user, who needs none.
pub const fn word(rung: TrustRung) -> Option<&'static str> {
    match rung {
        TrustRung::Blocked => Some("Blocked"),
        TrustRung::Anonymous => Some("Anonymous"),
        TrustRung::Guest => Some("Guest"),
        TrustRung::Trusted => Some("Trusted"),
        TrustRung::Myself => None,
    }
}
