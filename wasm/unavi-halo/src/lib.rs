//! Halo: the personal shell.
//!
//! A tree of motes, summoned from anywhere, that reaches what is yours — your
//! tools, and the places you can go — and nothing belonging to the space you
//! happen to be standing in.
//!
//! Everything in the halo itself is `unavi:vui`'s: halo builds motes, mounts an
//! orbit over them, and reads back what happened, holding no prim and doing no
//! hit-testing. What it draws for itself is only what VUI has no business
//! knowing about — the glyphs inside its motes, and the body of whatever tool
//! is in hand.

use std::time::SystemTime;

use wired_prelude::prelude::*;

use crate::{
    artifact::Artifact,
    branch::{
        hand::Hand,
        home::Home,
        places::Places,
    },
    root::Root,
    summon::{
        Command,
        Summon,
    },
    unavi::vui::api::{
        self,
        Event,
        Mote,
    },
    wired::{
        agent::api::local_camera,
        input::{
            context::register_global_input_listener,
            types::{
                InputAction,
                InputListener,
            },
        },
        scene::types::Prim,
    },
};

mod artifact;
mod branch;
mod icon;
mod palette;
mod root;
mod summon;

wired_prelude::generate_script!(Script);

struct Script {
    root:     Root,
    hand:     Hand,
    home:     Home,
    places:   Places,
    summon:   Summon,
    /// The body of whatever is in hand, and the physgun's muzzle.
    artifact: Artifact,
    /// The menu button, which is the one thing halo takes globally. Grabs
    /// belong to whichever surface was pressed, and VUI's surfaces have their
    /// own listeners.
    input:    InputListener,
    camera:   Option<Prim>,
    drawn_at: SystemTime,
}

impl Script {
    fn eye(&mut self) -> Option<Transform> {
        if self.camera.is_none() {
            self.camera = local_camera().ok();
        }
        self.camera.as_ref().map(Prim::global_xform)
    }

    fn apply(&mut self, command: Command) -> anyhow::Result<()> {
        match command {
            Command::Summon => {
                self.root.orbit.summon()?;
                self.hand.unequip();
            }
            Command::Dismiss => self.root.orbit.dismiss()?,
            Command::None => {}
        }
        Ok(())
    }

    /// Routes one thing a surface did, by the mote it happened to. Never by a
    /// label: two motes with the same name are still two motes.
    fn route(&mut self, event: &Event, eye: &Transform) -> anyhow::Result<()> {
        match event {
            Event::Opened(mote) => self.opened(mote),
            Event::Cast(mote) => self.cast(mote),
            Event::Activated(mote) => self.activated(mote, eye)?,
            Event::Planted((mote, landing)) => {
                // Planting from the halo puts it away: attention has moved to
                // the thing that was just placed.
                if self.places.plant(mote, *landing) {
                    let dismiss = self.summon.taken();
                    self.apply(dismiss)?;
                }
            }
            Event::Closed(_) | Event::Casting(_) | Event::Aborted(_) | Event::Filed(_) => {}
            Event::Paged(page) => {
                println!("halo: page {} of {}", page.index + 1, page.count);
            }
        }
        Ok(())
    }

    /// A branch is filled when it opens rather than up front, so a halo nobody
    /// opens costs nothing and an unbounded level is never walked.
    fn opened(&mut self, mote: &Mote) {
        if mote.is(&self.root.places) {
            self.places.refresh();
        }
    }

    fn cast(&mut self, mote: &Mote) {
        if mote.is(&self.root.home) {
            self.home.request();
        } else {
            self.places.cast(mote);
        }
    }

    fn activated(&mut self, mote: &Mote, eye: &Transform) -> anyhow::Result<()> {
        if !self.hand.equip(mote, eye) {
            return Ok(());
        }
        if let Some(color) = self.hand.held_color() {
            self.artifact.wear(color);
        }
        let dismiss = self.summon.taken();
        self.apply(dismiss)
    }

    /// The menu button, and the primary action while the halo is down.
    fn read_input(&mut self, eye: &Transform) -> anyhow::Result<()> {
        while let Some(event) = self.input.poll() {
            match event.action {
                InputAction::MenuDown => {
                    let command = self.summon.press(eye);
                    self.apply(command)?;
                }
                InputAction::MenuUp => self.summon.release(),
                // Forwarded only while the halo is down. With it up, a press
                // belongs to whichever surface was pressed, and VUI's own
                // listener is what hears it.
                InputAction::GrabDown => self.forward(|hand| hand.trigger(true)),
                InputAction::GrabUp => self.forward(|hand| hand.trigger(false)),
                InputAction::ScrollUp => self.forward(|hand| hand.scroll(1.0)),
                InputAction::ScrollDown => self.forward(|hand| hand.scroll(-1.0)),
            }
        }
        Ok(())
    }

    fn forward(&self, to_hand: impl FnOnce(&Hand)) {
        if !self.summon.is_up() {
            to_hand(&self.hand);
        }
    }
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        Ok(Self {
            root:     Root::new()?,
            hand:     Hand::new(),
            home:     Home::default(),
            places:   Places::default(),
            summon:   Summon::default(),
            artifact: Artifact::new()?,
            input:    register_global_input_listener()?,
            camera:   None,
            drawn_at: SystemTime::now(),
        })
    }

    fn fixed_update(&mut self) -> anyhow::Result<()> {
        api::fixed_update()?;
        self.home.fixed_update();
        self.hand.fixed_update(&self.root.tools);
        self.places.fixed_update(&self.root.places);

        let Some(eye) = self.eye() else {
            return Ok(());
        };
        self.read_input(&eye)?;

        let walked_away = self.summon.step(&eye);
        self.apply(walked_away)
    }

    fn update(&mut self) -> anyhow::Result<()> {
        api::update()?;
        // Animation runs at render rate; pinning it to the fixed rate makes
        // motion visibly step.
        let delta = self.drawn_at.elapsed().unwrap_or_default().as_secs_f32();
        self.drawn_at = SystemTime::now();

        let Some(eye) = self.eye() else {
            return Ok(());
        };
        for event in self.root.orbit.events() {
            self.route(&event, &eye)?;
        }
        self.artifact.update(&eye, self.hand.is_holding(), delta)
    }
}
