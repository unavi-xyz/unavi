//! Visor: the shell the user perceives through.
//!
//! Where the halo is what the user acts through, the visor draws what the
//! world is not saying for itself: for now, a placard over each portal being
//! looked at. It is a registry of lenses, each a layer that is worn or not,
//! announced to the halo so the halo can offer them.
//!
//! The visor draws only through `unavi:vui`, in its own document, at the
//! user's own trust.

use unavi_lens::TRAVEL;

use crate::{
    lens::Registry,
    protocol::{
        Channel,
        Request,
    },
    travel::Travel,
    unavi::vui::api,
};

mod lens;
mod protocol;
mod travel;

wired_guest::generate_script!(Script);

struct Script {
    registry: Registry,
    channel:  Channel,
    travel:   Travel,
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let mut script = Self {
            registry: Registry::default(),
            channel:  Channel::default(),
            travel:   Travel::new()?,
        };
        script.channel.start();
        Channel::announce(script.registry.lenses());
        Ok(script)
    }

    fn fixed_update(&mut self, _tick: Tick) -> anyhow::Result<()> {
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

        self.travel
            .fixed_update(self.registry.is_worn(TRAVEL), camera_pose().as_ref());
        api::fixed_update()?;
        Ok(())
    }

    fn update(&mut self, tick: Tick) -> anyhow::Result<()> {
        api::update(tick.dt)?;
        Ok(())
    }
}
