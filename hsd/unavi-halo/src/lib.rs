//! Halo: the personal shell.
//!
//! A tree of motes, summoned from anywhere, that reaches what is yours — your
//! tools, and the spaces you can go to — and nothing belonging to the one you
//! happen to be standing in.
//!
//! Everything in the halo itself is `unavi:vui`'s: halo builds motes, mounts an
//! orbit over them, and reads back what happened, holding no prim and doing no
//! hit-testing. What it draws for itself is only what VUI has no business
//! knowing about — the glyphs inside its motes, and the body of whatever tool
//! is in hand.

use wired_guest::{
    math::Transform,
    teleport::Watch,
};

use crate::{
    artifact::Artifact,
    branch::{
        hand::Toolbelt,
        home::Home,
        nav::{
            Nav,
            Planting,
        },
    },
    clock::Clock,
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
    wired::input::{
        device,
        types::{
            Action,
            Button,
            InputSubscription,
        },
    },
};

mod artifact;
mod branch;
mod clock;
mod glyph;
mod palette;
mod root;
mod summon;

wired_guest::generate_script!(Script);

struct Script {
    root:     Root,
    hand:     Toolbelt,
    home:     Home,
    nav:      Nav,
    summon:   Summon,
    /// Last tick's eye, so a respawn carries the open halo along.
    jumps:    Watch,
    /// The time on the root's face.
    clock:    Clock,
    /// The body of whatever is in hand, and the physgun's muzzle.
    artifact: Artifact,
    /// The menu button, which is the one thing halo takes globally. Grabs
    /// belong to whichever surface was pressed, and VUI's surfaces have their
    /// own listeners.
    input:    InputSubscription,
}

impl Script {
    /// Turns the root's face over when the minute does. Written while the
    /// halo is down too, so it is already right the moment it is summoned.
    fn show_time(&mut self) {
        let Some(text) = self.clock.tick() else {
            return;
        };
        if let Err(err) = self.root.orbit.set_readout(Some(&text)) {
            eprintln!("halo: readout: {err:?}");
        }
    }

    fn apply(&self, command: Option<Command>) -> anyhow::Result<()> {
        match command {
            // Whatever is in hand stays in hand: a tool is put away by
            // choosing it again or choosing another, never by looking at the
            // halo. Opening a menu is not a decision about what you are
            // holding.
            Some(Command::Summon) => self.root.orbit.summon()?,
            Some(Command::Dismiss) => self.root.orbit.dismiss()?,
            None => {}
        }
        Ok(())
    }

    /// Routes one thing a surface did, by the mote it happened to. Never by a
    /// label: two motes with the same name are still two motes.
    fn route(&mut self, event: &Event) -> anyhow::Result<()> {
        match event {
            Event::Opened(mote) => self.opened(mote),
            Event::Cast(mote) => self.cast(mote),
            Event::Activated(mote) => self.activated(mote)?,
            Event::Planted(planted) => {
                // Planting from the halo puts it away: attention has moved to
                // the thing that was just placed.
                if matches!(
                    self.nav.plant(&planted.mote, planted.landing),
                    Planting::Planted
                ) {
                    let dismiss = self.summon.taken();
                    self.apply(dismiss)?;
                }
            }
            Event::Closed(_) | Event::Casting(_) | Event::Aborted(_) | Event::Filed(_) => {}
            Event::Paged(page) => {
                println!("halo: page {} of {}", page.index + 1, page.pages);
            }
        }
        Ok(())
    }

    /// A branch is filled when it opens rather than up front, so a halo nobody
    /// opens costs nothing and an unbounded level is never walked.
    fn opened(&mut self, mote: &Mote) {
        if mote.is(&self.root.nav) {
            self.nav.refresh();
        }
    }

    fn cast(&mut self, mote: &Mote) {
        if mote.is(&self.root.home) {
            self.home.request(&self.root.home);
        } else {
            self.nav.cast(mote);
        }
    }

    fn activated(&mut self, mote: &Mote) -> anyhow::Result<()> {
        if !self.hand.equip(mote) {
            return Ok(());
        }
        if let Some(color) = self.hand.held_color() {
            self.artifact.wear(color);
        }
        let dismiss = self.summon.taken();
        self.apply(dismiss)
    }

    /// The menu button, and the primary action while the halo is down.
    ///
    /// A failed summon is reported rather than returned: the events drained
    /// with it are gone from the queue, and dropping the menu's release would
    /// latch the button down and ignore every press after it.
    fn read_input(&mut self, eye: &Transform) {
        for event in self.input.drain(8) {
            match event.action {
                Action::Pressed(Button::Menu) => {
                    let command = self.summon.press(eye);
                    if let Err(err) = self.apply(command) {
                        eprintln!("halo: menu: {err:?}");
                    }
                }
                Action::Released(Button::Menu) => self.summon.release(),
                // Forwarded only while the halo is down. With it up, a press
                // belongs to whichever surface was pressed, and VUI's own
                // listener is what hears it.
                Action::Pressed(Button::Trigger) => self.forward(|hand| hand.trigger(true)),
                Action::Released(Button::Trigger) => self.forward(|hand| hand.trigger(false)),
                Action::Scroll(delta) => self.forward(|hand| hand.scroll(delta.y)),
                // The grip is the host's, for carrying things about: a tool is
                // worked with its trigger, so an equipped one hears nothing of
                // it and the two never answer the same press. Hover is
                // per-prim, so a global listener never sees that either.
                Action::Pressed(Button::Grip)
                | Action::Released(Button::Grip)
                | Action::Entered
                | Action::Left => {}
            }
        }
    }

    fn forward(&self, to_hand: impl FnOnce(&Toolbelt)) {
        if !self.summon.is_up() {
            to_hand(&self.hand);
        }
    }
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        Ok(Self {
            root:     Root::new()?,
            hand:     Toolbelt::new(),
            home:     Home::default(),
            nav:      Nav::default(),
            summon:   Summon::default(),
            jumps:    Watch::default(),
            clock:    Clock::default(),
            artifact: Artifact::new()?,
            input:    device::listen()?,
        })
    }

    fn fixed_update(&mut self, _tick: Tick) -> anyhow::Result<()> {
        api::fixed_update()?;
        self.home.fixed_update(&self.root.home);
        self.hand.fixed_update(&self.root.tools);
        self.nav.fixed_update(&self.root.nav);
        self.show_time();

        let Some(eye) = camera_pose() else {
            return Ok(());
        };
        if let Some(by) = self.jumps.step(&eye) {
            self.summon.carry(&by);
        }
        self.read_input(&eye);

        let walked_away = self.summon.step(&eye);
        self.apply(walked_away)
    }

    fn update(&mut self, tick: Tick) -> anyhow::Result<()> {
        api::update(tick.dt)?;

        let Some(eye) = camera_pose() else {
            return Ok(());
        };
        for event in self.root.orbit.events() {
            self.route(&event)?;
        }
        self.artifact.update(&eye, self.hand.is_holding(), tick.dt)
    }
}
