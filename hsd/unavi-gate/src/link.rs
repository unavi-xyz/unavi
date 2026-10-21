//! Keeps the portal leading where the seated beacon does, answers portals
//! elsewhere asking for their far half, and lets go of a pairing whose far
//! half is gone or that someone pulled out of the socket.

use wired_guest::beacon::DocumentId;

use crate::{
    seat::Occupant,
    wired::{
        portal::portals::{
            self,
            IntentSubscription,
            LinkStatus,
        },
        scene::{
            document::Document,
            properties::{
                LinkId,
                Property,
                PropertyKey,
            },
        },
    },
};

/// Fixed ticks between opens while the portal does not yet lead where the
/// seated beacon does, as when an open was refused for quota.
const RETRY_TICKS: u32 = 32;

/// Fixed ticks after an unplug during which that link is refused even though
/// it is asked for again. The far side is still leading while it has not yet
/// seen the socket empty, so its re-ask is the same pairing, not a new seat.
/// Longer than the far side's retry interval, so the re-ask never wins.
const UNPLUG_GRACE_TICKS: u32 = 64;

/// The custom property on the portal prim naming the link last pulled out of
/// the socket, as 16 little-endian bytes, and empty once a new seating has
/// spent it. Any peer may state it, since it is in the `state` group.
const UNPLUGGED_KEY: &str = "state/unplugged";

fn link_bytes(link: LinkId) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(16);
    bytes.extend_from_slice(&link.0.to_le_bytes());
    bytes.extend_from_slice(&link.1.to_le_bytes());
    bytes
}

fn link_from(bytes: &[u8]) -> Option<LinkId> {
    let low: [u8; 8] = bytes.get(..8)?.try_into().ok()?;
    let high: [u8; 8] = bytes.get(8..16)?.try_into().ok()?;
    Some((u64::from_le_bytes(low), u64::from_le_bytes(high)))
}

/// What this peer has made of the portal since the socket was last empty.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Opened {
    /// Nothing: the portal leads where someone else put it.
    No,
    /// Opened or answered it, and no failed open reported yet.
    Unreported,
    /// Opened or answered it, and a failed open already reported.
    Reported,
}

pub struct Link {
    portal:      (u64, u64),
    intent_rx:   IntentSubscription,
    /// The space the portal leads to, and the link it bears there.
    destination: Option<(DocumentId, Option<LinkId>)>,
    /// What this peer has made of the portal since the socket was last empty,
    /// so emptying it closes what this peer opened.
    opened:      Opened,
    retry_in:    u32,
    /// Whether the destination has read as paired since it last changed.
    seen_paired: bool,
    /// The unplugged link this peer has already closed its half of.
    released:    Option<LinkId>,
    /// Whether the unplugged mark is already claimed, so the next ask for it
    /// spends it rather than being refused.
    claimed:     bool,
    /// Fixed ticks left in which the unplugged link is refused outright.
    grace:       u32,
    /// The seated beacon whose link the far side pulled out.
    revoked:     Option<DocumentId>,
}

impl Link {
    pub fn new(portal: (u64, u64)) -> anyhow::Result<Self> {
        Ok(Self {
            portal,
            intent_rx: portals::intents()?,
            destination: None,
            opened: Opened::No,
            retry_in: 0,
            seen_paired: false,
            released: None,
            claimed: false,
            grace: 0,
            revoked: None,
        })
    }

    /// The space the portal opens onto.
    pub fn leads(&self) -> Option<DocumentId> {
        self.destination.map(|(space, _)| space)
    }

    pub fn update(&mut self, doc: &Document, occupant: Option<Occupant>) {
        let destination = match doc.get(self.portal, &PropertyKey::Portal) {
            Some(Property::Portal(portal)) => portal.destination.map(|d| (d.space, d.link)),
            _ => None,
        };
        if destination != self.destination {
            self.destination = destination;
            self.seen_paired = false;
            self.released = None;
        }
        if self.unplugged(doc).is_none() {
            self.claimed = false;
            self.grace = 0;
        } else {
            self.grace = self.grace.saturating_sub(1);
        }
        self.follow_seat(doc, occupant);
        match occupant {
            None => self.let_go(doc),
            Some(occupant) => self.check_revoked(doc, occupant),
        }
        self.answer_intents(doc, occupant);
    }

    /// Pulls the pairing out: states its link as unplugged, which every
    /// peer's gate reads and lets go of, and closes this peer's own half.
    /// The mark holds for [`UNPLUG_GRACE_TICKS`] even though it is claimed
    /// already, since the far side keeps leading until it sees the socket
    /// empty.
    pub fn unplug(&mut self, doc: &Document) -> anyhow::Result<()> {
        let Some(link) = self.destination.and_then(|(_, link)| link) else {
            return Ok(());
        };
        doc.shared()
            .set(
                self.portal,
                Property::Custom((UNPLUGGED_KEY.to_owned(), link_bytes(link))),
            )
            .flush()?;
        self.released = Some(link);
        self.claimed = true;
        self.grace = UNPLUG_GRACE_TICKS;
        self.close(doc);
        Ok(())
    }

    fn unplugged(&self, doc: &Document) -> Option<LinkId> {
        match doc.get(self.portal, &PropertyKey::Custom(UNPLUGGED_KEY.to_owned())) {
            Some(Property::Custom((_, bytes))) => link_from(&bytes),
            _ => None,
        }
    }

    /// Empties the unplugged mark if it still names `link`.
    fn clear_unplugged(&self, doc: &Document, link: LinkId) {
        if self.unplugged(doc) != Some(link) {
            return;
        }
        let cleared = doc
            .shared()
            .set(
                self.portal,
                Property::Custom((UNPLUGGED_KEY.to_owned(), Vec::new())),
            )
            .flush();
        if let Err(err) = cleared {
            eprintln!("gate: clear unplugged: {err:?}");
        }
    }

    fn close(&self, doc: &Document) {
        if let Err(err) = portals::close(doc, self.portal) {
            eprintln!("gate: close: {err:?}");
        }
    }

    /// Keeps the portal leading where the seated beacon does, reopening every
    /// [`RETRY_TICKS`] while it does not, and closes what this peer opened once
    /// the socket is empty.
    ///
    /// A portal already leading to the seated beacon's space is left as it
    /// is, since a second open states a different link and would break the
    /// pair the portal is half of. Should whoever stated it leave, the
    /// destination changes and this peer opens it again.
    fn follow_seat(&mut self, doc: &Document, occupant: Option<Occupant>) {
        // Nothing seated: this gate leads nowhere, whatever opened it. The
        // close is idempotent, so it is stated even when a revoke already
        // closed this peer's half; otherwise taking the beacon out after the
        // far half pulled would leave the portal leading its last space.
        let Some(occupant) = occupant else {
            self.retry_in = 0;
            self.revoked = None;
            if self.opened != Opened::No {
                self.opened = Opened::No;
                self.close(doc);
            }
            return;
        };
        if self.revoked.is_some_and(|beacon| beacon != occupant.beacon) {
            self.revoked = None;
        }
        if self.revoked == Some(occupant.beacon) {
            return;
        }
        if self.leads() == Some(occupant.space) {
            return;
        }
        if self.retry_in > 0 {
            self.retry_in -= 1;
            return;
        }
        self.retry_in = RETRY_TICKS;
        if self.opened != Opened::Reported {
            self.opened = Opened::Unreported;
        }
        if let Err(err) = portals::open(doc, self.portal, occupant.space)
            && self.opened == Opened::Unreported
        {
            self.opened = Opened::Reported;
            eprintln!("gate: open: {err:?}");
        }
    }

    /// Drops this peer's pairing once it was pulled out of the socket, or
    /// once the portal that asked for it is gone: the destination still
    /// bears the link, but nothing there answers it.
    fn let_go(&mut self, doc: &Document) {
        let Some(link) = self.destination.and_then(|(_, link)| link) else {
            return;
        };
        if self.unplugged(doc) == Some(link) {
            if self.released != Some(link) {
                self.released = Some(link);
                self.claimed = false;
                self.close(doc);
            }
            return;
        }
        match portals::link_state(doc, self.portal) {
            Ok(LinkStatus::Paired) => self.seen_paired = true,
            Ok(LinkStatus::Unpaired) if self.seen_paired => {
                self.seen_paired = false;
                self.close(doc);
            }
            // Any other state, or not yet resolved this frame, is read again
            // next tick.
            Ok(_) | Err(_) => {}
        }
    }

    /// Drops the link when the far side pulled its half out, even though the
    /// beacon stays seated here. The socket keeps no destination until the
    /// beacon leaves, so the far side gets a fresh pair on the next seating.
    fn check_revoked(&mut self, doc: &Document, occupant: Occupant) {
        if self.revoked == Some(occupant.beacon) {
            return;
        }
        match portals::link_state(doc, self.portal) {
            Ok(LinkStatus::Paired) => self.seen_paired = true,
            Ok(status) if revokes(status, self.seen_paired) => {
                self.seen_paired = false;
                self.revoked = Some(occupant.beacon);
                self.close(doc);
            }
            // Not paired yet, or not resolved this frame, is read again next
            // tick.
            Ok(_) | Err(_) => {}
        }
    }

    /// Answers a portal elsewhere asking for its far half. An ask for the
    /// unplugged link is held off as described below; any other ask is let go
    /// while the socket is occupied or the gate leads somewhere, and paired
    /// otherwise.
    ///
    /// The first ask after an unplug is refused and claimed, so no other gate
    /// here answers it. The host offers it again only once the source withdrew
    /// the link, so a second ask is a beacon seated there afresh: it spends the
    /// mark and pairs. The claim is what makes that hold; the grace is what
    /// stops a re-ask that races the socket emptying from counting as one.
    fn answer_intents(&mut self, doc: &Document, occupant: Option<Occupant>) {
        for intent in self.intent_rx.drain(8) {
            // The mark is stated this tick, and the far side's re-ask can
            // arrive before it has seen the socket empty. Refusing and
            // claiming it holds the pairing shut for the grace, so it cannot
            // snap back open while the ghost is still dissolving.
            if held_off(self.unplugged(doc), intent.link, self.grace) {
                let _ = self.intent_rx.claim(intent);
                continue;
            }
            match consider(self.unplugged(doc), intent.link, self.claimed) {
                Ask::Refuse => {
                    self.claimed = true;
                    let _ = self.intent_rx.claim(intent);
                    continue;
                }
                Ask::Spend => {
                    self.claimed = false;
                    self.released = None;
                    self.clear_unplugged(doc, intent.link);
                }
                Ask::Answer => {}
            }
            if occupant.is_some() || self.destination.is_some() {
                continue;
            }
            if !self.intent_rx.claim(intent) {
                continue;
            }
            if let Err(err) = portals::pair(doc, self.portal, intent) {
                eprintln!("gate: pair: {err:?}");
            }
        }
    }
}

/// Whether an ask lands inside the unplug grace, where the mark is refused
/// whatever the far side has done since.
fn held_off(marked: Option<LinkId>, link: LinkId, grace: u32) -> bool {
    grace > 0 && marked == Some(link)
}

/// What an ask for `link` means while `marked` is unplugged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ask {
    /// The source's immediate re-ask after the unplug.
    Refuse,
    /// A new connection on the same link, so the mark is spent.
    Spend,
    /// Unrelated to the mark.
    Answer,
}

fn consider(marked: Option<LinkId>, link: LinkId, claimed: bool) -> Ask {
    if marked != Some(link) {
        return Ask::Answer;
    }
    if claimed { Ask::Spend } else { Ask::Refuse }
}

/// Whether losing a partner revokes the link. `Unpaired` also reads while the
/// far half has not appeared yet, so only a link that was paired counts.
const fn revokes(status: LinkStatus, seen_paired: bool) -> bool {
    seen_paired && matches!(status, LinkStatus::Unpaired)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unplugged_link_reads_back_as_itself() {
        let link = (0x0123_4567_89AB_CDEF, 0xFEDC_BA98_7654_3210);
        assert_eq!(link_from(&link_bytes(link)), Some(link));
        assert_eq!(link_from(&[]), None, "an empty value names no link");
    }

    const LINK: LinkId = (1, 2);
    const OTHER: LinkId = (3, 4);

    #[test]
    fn the_unplug_grace_holds_the_link_off_however_it_is_asked() {
        assert!(held_off(Some(LINK), LINK, 1));
        assert!(!held_off(Some(LINK), LINK, 0), "over, so it can be spent");
        assert!(!held_off(Some(OTHER), LINK, 64), "a different link");
        assert!(!held_off(None, LINK, 64), "nothing was unplugged");
    }

    #[test]
    fn only_the_unplugged_link_is_held_off() {
        assert_eq!(consider(None, LINK, false), Ask::Answer);
        assert_eq!(consider(Some(OTHER), LINK, false), Ask::Answer);
    }

    #[test]
    fn the_first_ask_after_an_unplug_is_refused_and_the_next_is_a_new_seat() {
        assert_eq!(consider(Some(LINK), LINK, false), Ask::Refuse);
        assert_eq!(consider(Some(LINK), LINK, true), Ask::Spend);
    }

    #[test]
    fn only_a_paired_link_can_be_revoked() {
        assert!(revokes(LinkStatus::Unpaired, true));
        assert!(!revokes(LinkStatus::Unpaired, false), "not paired yet");
        assert!(!revokes(LinkStatus::Paired, true));
        assert!(!revokes(LinkStatus::Pending, true));
        assert!(!revokes(LinkStatus::OneWay, true));
    }
}
