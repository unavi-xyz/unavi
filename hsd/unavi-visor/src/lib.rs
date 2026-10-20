//! Visor: the shell the user perceives through.
//!
//! Where the halo is what the user acts through, the visor draws what the
//! world is not saying for itself: a placard over each portal being looked
//! at, and a nameplate over each person. It is a registry of lenses, each a
//! layer that is worn or not, announced to the halo so the halo can offer
//! them.
//!
//! The visor draws only through `unavi:vui`, in its own document, at the
//! user's own trust.

use unavi_lens::{
    SOCIAL,
    TRAVEL,
};

use crate::{
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
mod lens;
mod protocol;
mod rung;
mod slots;
mod social;
mod travel;

wired_guest::generate_script!(Script);

struct Script {
    registry: Registry,
    channel:  Channel,
    travel:   Travel,
    social:   Social,
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let mut script = Self {
            registry: Registry::default(),
            channel:  Channel::default(),
            travel:   Travel::new()?,
            social:   Social::new()?,
        };
        script.channel.start();
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

        let eye = camera_pose();
        self.travel
            .fixed_update(self.registry.is_worn(TRAVEL), eye.as_ref());
        self.social
            .fixed_update(self.registry.is_worn(SOCIAL), eye.as_ref());
        api::fixed_update()?;
        Ok(())
    }

    fn update(&mut self, tick: Tick) -> anyhow::Result<()> {
        self.travel.update(camera_pose().as_ref());
        self.social.update();
        api::update(tick.dt)?;
        Ok(())
    }
}
