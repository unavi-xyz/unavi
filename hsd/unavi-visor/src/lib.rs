//! Visor: the shell the user perceives through.
//!
//! Where the halo is what the user acts through, the visor draws what the
//! world is not saying for itself: a tag beside each portal being looked
//! at, and a nameplate over each person. It is a registry of lenses, each a
//! layer that is worn or not, announced to the halo so the halo can offer
//! them.
//!
//! The visor draws only through `unavi:vui`, in its own document, at the
//! user's own trust.

use unavi_lens::{
    DOCUMENTS,
    SOCIAL,
    TRAVEL,
};

use crate::{
    documents::Documents,
    holds::Holds,
    lens::Registry,
    protocol::{
        Channel,
        Request,
    },
    social::Social,
    travel::Travel,
    unavi::vui::api,
};

mod attention;
mod clearance;
mod documents;
mod holds;
mod lens;
mod protocol;
mod rung;
mod slots;
mod social;
mod tag;
mod travel;

wired_guest::generate_script!(Script);

struct Script {
    registry:  Registry,
    channel:   Channel,
    travel:    Travel,
    social:    Social,
    documents: Documents,
    holds:     Holds,
    /// Fixed ticks until holds are checked for documents that unloaded.
    sweep_in:  u32,
}

/// Fixed ticks between checks that the documents holding a lens still exist,
/// about a second.
const SWEEP_TICKS: u32 = 64;

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let mut script = Self {
            registry:  Registry::default(),
            channel:   Channel::default(),
            travel:    Travel::new()?,
            social:    Social::new()?,
            documents: Documents::new()?,
            holds:     Holds::default(),
            sweep_in:  SWEEP_TICKS,
        };
        script.channel.start();
        script.holds.start();
        Channel::announce(script.registry.lenses());
        Ok(script)
    }

    fn fixed_update(&mut self, _tick: Tick) -> anyhow::Result<()> {
        self.travel.set_shell(self.channel.shell());
        for request in self.channel.poll() {
            match request {
                Request::Ask => Channel::announce(self.registry.lenses()),
                Request::Wear(wear) => {
                    if self.registry.wear(&wear) {
                        Channel::announce(self.registry.lenses());
                    }
                }
            }
        }

        for (sender, hold) in self.holds.poll() {
            self.registry.hold(sender, &hold);
        }
        if self.sweep_in == 0 {
            self.sweep_in = SWEEP_TICKS;
            self.registry.release_unloaded(holds::is_loaded);
        }
        self.sweep_in -= 1;

        let eye = camera_pose();
        self.travel
            .fixed_update(self.registry.is_shown(TRAVEL), eye.as_ref());
        self.social
            .fixed_update(self.registry.is_shown(SOCIAL), eye.as_ref());
        self.documents
            .fixed_update(self.registry.is_shown(DOCUMENTS), eye.as_ref());
        api::fixed_update()?;
        Ok(())
    }

    fn update(&mut self, tick: Tick) -> anyhow::Result<()> {
        self.travel.update(camera_pose().as_ref());
        self.social.update();
        self.documents.update();
        api::update(tick.dt)?;
        Ok(())
    }
}
