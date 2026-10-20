//! How a trust rung reads on a placard.

use crate::wired::perception::perception::TrustRung;

pub const fn word(rung: TrustRung) -> &'static str {
    match rung {
        TrustRung::Blocked => "Blocked",
        TrustRung::Anonymous => "Anonymous",
        TrustRung::Guest => "Guest",
        TrustRung::Trusted => "Trusted",
        TrustRung::Myself => "Yours",
    }
}
