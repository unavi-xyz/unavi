//! Halo: the personal shell.
//!
//! A tree of motes, summoned from anywhere, that reaches what is yours — your
//! tools, the spaces you can go to, and the lenses you see the world through —
//! and nothing belonging to the one you happen to be standing in.
//!
//! Everything in the halo itself is `unavi:vui`'s: halo builds motes, mounts a
//! bed of keys over them, and reads back what happened, holding no prim and
//! doing no hit-testing. What it draws for itself is only what VUI has no
//! business knowing about — the glyphs inside its motes, and the body of
//! whatever tool is in hand.

use wired_guest::{
    math::Transform,
    xform::tool_pose,
};

use crate::{
    artifact::Artifact,
    body::Body,
    branch::{
        hand::Toolbelt,
        home::Home,
        nav::{
            Nav,
            Planting,
        },
        visor::Visor,
    },
    notice::Notice,
    page::{
        Page,
        Rest,
    },
    root::Root,
    shortcuts::{
        Shortcut,
        Shortcuts,
    },
    summon::{
        Command,
        Residence,
        Summon,
    },
    unavi::{
        author::api::{
            History,
            Step,
            Way,
        },
        vui::api::{
            self,
            Event,
            Mote,
        },
    },
    wired::{
        input::{
            device,
            types::{
                Action,
                Button,
                InputSubscription,
                PointerKind,
            },
        },
        portal::portals::current_space,
    },
};

mod artifact;
mod body;
mod branch;
mod clock;
mod glyph;
mod notice;
mod page;
mod palette;
mod root;
mod shortcuts;
mod summon;

wired_guest::generate_script!(Script);

struct Script {
    root:      Root,
    hand:      Toolbelt,
    home:      Home,
    nav:       Nav,
    /// The visor script's lenses, mirrored as toggles.
    visor:     Visor,
    summon:    Summon,
    /// Attaches the halo's document to the user's body.
    body:      Body,
    /// The space last tick, so a respawn, travel or home can close the halo.
    space:     Residence,
    /// What the root's page says while no key is attended.
    page:      Page,
    /// The body of whatever is in hand, and the physgun's muzzle.
    artifact:  Artifact,
    /// The menu button, and the buttons halo forwards to the tool in hand.
    /// Presses on a surface belong to it, and VUI's surfaces have their own
    /// listeners.
    input:     InputSubscription,
    /// The pointer that last pressed a button: the hand a tool chosen now
    /// goes into.
    aim:       PointerKind,
    /// Every edit the local user's tools kept, to undo and redo.
    history:   History,
    shortcuts: Shortcuts,
    /// Says what an undo or redo did.
    notice:    Notice,
}

impl Script {
    /// Keeps the root's page current: the time, and where you are. Written
    /// while the halo is down too, so it is already right the moment it is
    /// summoned.
    fn show_page(&mut self) {
        let rest = Rest::of(clock::time(), &self.nav.presence());
        self.page.show(&self.root.orbit, rest);
    }

    fn apply(&self, command: Option<Command>) -> anyhow::Result<()> {
        match command {
            // Whatever is in hand stays in hand: a tool is put away by
            // choosing it again or choosing another, never by looking at the
            // halo. Opening a menu is not a decision about what you are
            // holding.
            Some(Command::Summon) => {
                self.hand.lift();
                self.root.orbit.summon()?;
            }
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
            Event::Cast(mote) => self.cast(mote)?,
            Event::Activated(mote) => self.activated(mote)?,
            Event::Planted(planted) => {
                // Planting from the halo puts it away: attention has moved to
                // the thing that was just placed.
                if matches!(
                    self.nav.plant(&planted.mote, planted.landing),
                    Planting::Planted
                ) {
                    self.put_away()?;
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
        } else if mote.is(&self.root.me) || mote.is(&self.root.visor) {
            Visor::ask();
        }
    }

    /// A journey begun puts the halo away at once: left up, it stands through
    /// the whole departure and arrival with the cast that sent it.
    fn cast(&mut self, mote: &Mote) -> anyhow::Result<()> {
        if mote.is(&self.root.home) {
            self.home.request(&self.root.home);
        } else if self.nav.cast(mote) {
            self.put_away()?;
        }
        Ok(())
    }

    fn put_away(&mut self) -> anyhow::Result<()> {
        let dismiss = self.summon.taken();
        self.apply(dismiss)
    }

    fn activated(&mut self, mote: &Mote) -> anyhow::Result<()> {
        // A lens is a switch, not a decision to be done with the halo: it stays
        // up so several can be set in one visit.
        if self.visor.wear(mote) {
            return Ok(());
        }
        // Undo and redo leave the halo up, so several steps are one visit.
        let way = if mote.is(&self.root.undo) {
            Some(Way::Undo)
        } else if mote.is(&self.root.redo) {
            Some(Way::Redo)
        } else {
            None
        };
        if let Some(way) = way {
            let label = match way {
                Way::Undo => self.history.undo(),
                Way::Redo => self.history.redo(),
            };
            self.notice.step(&Step { way, label });
            return Ok(());
        }
        if !self.hand.equip(mote, self.aim) {
            return Ok(());
        }
        if let Some(color) = self.hand.held_color() {
            self.artifact.wear(color);
        }
        self.put_away()
    }

    /// The menu button, and every button and key while the halo is down.
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
                // The hand that pressed last is the hand that chose a tool, so
                // it is noted whether or not the halo is up.
                Action::Pressed(button) => {
                    self.aim = event.pointer;
                    self.forward(|hand| {
                        hand.button(event.pointer, button, true, event.hit.as_ref());
                    });
                }
                // With the halo up a press belongs to whichever surface was
                // pressed, and VUI's own listener is what hears it.
                Action::Released(button) => {
                    self.forward(|hand| {
                        hand.button(event.pointer, button, false, event.hit.as_ref());
                    });
                }
                Action::KeyPressed(key) => {
                    self.hand.key(key, true, !self.summon.is_up());
                    let shortcut = self.shortcuts.key(key, true);
                    self.shortcut(shortcut);
                }
                Action::KeyReleased(key) => {
                    self.hand.key(key, false, !self.summon.is_up());
                    self.shortcuts.key(key, false);
                }
                Action::Scroll(delta) => self.forward(|hand| hand.scroll(delta.y)),
                // Hover is per-prim, so a global listener never sees it.
                Action::Entered | Action::Left => {}
            }
        }
    }

    fn shortcut(&mut self, shortcut: Option<Shortcut>) {
        let step = match shortcut {
            Some(Shortcut::Undo) => Step {
                way:   Way::Undo,
                label: self.history.undo(),
            },
            Some(Shortcut::Redo) => Step {
                way:   Way::Redo,
                label: self.history.redo(),
            },
            None => return,
        };
        self.notice.step(&step);
    }

    fn forward(&self, to_hand: impl FnOnce(&Toolbelt)) {
        if !self.summon.is_up() {
            to_hand(&self.hand);
        }
    }
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let mut script = Self {
            root:      Root::new()?,
            hand:      Toolbelt::new(),
            home:      Home::default(),
            nav:       Nav::default(),
            visor:     Visor::default(),
            summon:    Summon::default(),
            body:      Body::default(),
            space:     Residence::default(),
            page:      Page::default(),
            artifact:  Artifact::new()?,
            input:     device::listen()?,
            aim:       PointerKind::Screen,
            history:   History::new(),
            shortcuts: Shortcuts::default(),
            notice:    Notice::new()?,
        };
        // Listening before asking, so the answer cannot arrive unheard.
        script.visor.start();
        Visor::ask();
        Ok(script)
    }

    fn fixed_update(&mut self, _tick: Tick) -> anyhow::Result<()> {
        self.body.step()?;
        api::fixed_update()?;
        for step in self.history.poll() {
            self.notice.step(&step);
        }
        if self.home.fixed_update(&self.root.home) {
            self.put_away()?;
        }
        self.hand.fixed_update(&self.root.tools);
        self.nav.fixed_update(&self.root.nav);
        self.visor.fixed_update(&self.root.visor);
        self.show_page();

        // A respawn, a travel or home changes the space; a menu opened in the
        // one the user left no longer belongs to them.
        if self.space.step(current_space().ok().flatten()) {
            self.put_away()?;
        }

        let Some(eye) = camera_pose() else {
            return Ok(());
        };
        self.read_input(&eye);

        // Closing ends the grasp, so a mote in hand keeps the halo up.
        if self.root.orbit.held() {
            return Ok(());
        }
        let leaned_away = self.summon.step(&eye);
        self.apply(leaned_away)
    }

    fn update(&mut self, tick: Tick) -> anyhow::Result<()> {
        api::update(tick.dt)?;

        let Some(eye) = camera_pose() else {
            return Ok(());
        };
        for event in self.root.orbit.events() {
            self.route(&event)?;
        }
        self.notice.update(&eye, tick.dt);
        let pose = self
            .hand
            .hand()
            .and_then(held_tool_pose)
            .unwrap_or_else(|| tool_pose(&eye, None));
        self.artifact.update(&pose, self.hand.is_holding(), tick.dt)
    }
}
