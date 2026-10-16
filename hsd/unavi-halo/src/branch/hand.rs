//! Tools: what can be at hand, and which one is.
//!
//! Each tool is a toggle, so the mote is the switch: the one that is on burns
//! lit and the rest are dark, and there is nothing to keep in step with what
//! the user can see. One at a time is halo's rule rather than VUI's.

use wired_guest::math::Color;

use crate::{
    glyph,
    palette,
    unavi::{
        tool::api::{
            ToolRegistry,
            ToolState,
        },
        vui::api::{
            Kind,
            Mote,
        },
    },
};

const RESTING: Color = Color {
    r: 0.72,
    g: 0.76,
    b: 0.82,
    a: 1.0,
};

const HELD: Color = Color {
    r: 0.95,
    g: 0.93,
    b: 0.86,
    a: 1.0,
};

struct Tool {
    doc:  (u64, u64, u64, u64),
    name: String,
    mote: Mote,
}

pub struct Toolbelt {
    registry: ToolRegistry,
    tools:    Vec<Tool>,
    /// What is in the hand.
    held:     Option<(u64, u64, u64, u64)>,
}

impl Default for Toolbelt {
    fn default() -> Self {
        Self::new()
    }
}

impl Toolbelt {
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: ToolRegistry::new(),
            tools:    Vec::new(),
            held:     None,
        }
    }

    /// Picks up newly announced tools and hangs a mote for each under
    /// `parent`, ordered by a stable key rather than by the order they
    /// answered in — a level that reorders between frames silently rebinds
    /// muscle memory.
    pub fn fixed_update(&mut self, parent: &Mote) {
        let mut found = false;
        for tool in self.registry.poll() {
            if self.tools.iter().any(|held| held.doc == tool.document) {
                continue;
            }
            self.registry.set_state(tool.document, state(RESTING));

            // A toggle rather than something to carry out of the halo: with no
            // tracked hand to put a tool in, one that left its slot would be a
            // mode with nowhere visible to live. Holding a tool is a switch
            // until there is a hand to hold it in.
            // The tool says what it does; how to hold one is the same for all
            // of them, and the mote's own kind already says it is a switch.
            let mote = match Mote::new(Kind::Toggle, &tool.name) {
                Ok(mote) => mote,
                Err(err) => {
                    eprintln!("halo: tool mote for '{}': {err:?}", tool.name);
                    continue;
                }
            };
            if let Err(err) = mote.set_description(&tool.description) {
                eprintln!("halo: tool description for '{}': {err:?}", tool.name);
            }
            // The mark is the tool's own, else the generic one. A refused mark
            // leaves the mote undrawn rather than losing the tool.
            let mark = tool
                .glyph
                .map_or_else(|| glyph::from_rows(&glyph::TOOL), glyph::from_tool);
            if let Err(err) = mote.set_glyph(Some(&mark)) {
                eprintln!("halo: tool glyph for '{}': {err:?}", tool.name);
            }

            self.tools.push(Tool {
                doc: tool.document,
                name: tool.name,
                mote,
            });
            found = true;
        }
        if !found {
            return;
        }

        self.tools
            .sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.doc.cmp(&b.doc)));
        // Re-coloured after every sort rather than at construction: a tool's
        // colour is its place in the set, and the set is what just changed.
        for (index, tool) in self.tools.iter().enumerate() {
            tool.mote.set_tint(Some(palette::tool(index)));
            parent.add_child(&tool.mote);
        }
    }

    /// Whether `mote` is a tool, and takes it into the hand if it is.
    ///
    /// Choosing the tool already in hand puts it away, so one gesture both
    /// equips and unequips and there is no separate way to stop.
    pub fn equip(&mut self, mote: &Mote) -> bool {
        let Some(tool) = self.tools.iter().find(|tool| tool.mote.is(mote)) else {
            return false;
        };
        let doc = tool.doc;
        let name = tool.name.clone();

        if self.held == Some(doc) {
            self.unequip();
            return true;
        }
        self.unequip();

        println!("halo: holding '{name}'");
        self.registry.activate(doc);
        self.registry.set_state(doc, state(HELD));
        self.held = Some(doc);
        self.mark();
        true
    }

    /// Puts down whatever is in the hand.
    pub fn unequip(&mut self) {
        let Some(doc) = self.held.take() else {
            return;
        };
        self.registry.deactivate(doc);
        self.registry.set_state(doc, state(RESTING));
        self.mark();
    }

    #[must_use]
    pub const fn is_holding(&self) -> bool {
        self.held.is_some()
    }

    /// The colour of whatever is in hand, for anything that has to look like
    /// the tool it belongs to.
    #[must_use]
    pub fn held_color(&self) -> Option<Color> {
        let doc = self.held?;
        let index = self.tools.iter().position(|tool| tool.doc == doc)?;
        Some(palette::tool(index))
    }

    /// Lights the mote of whatever is in hand and puts every other one out.
    ///
    /// A toggle flips itself when chosen; only one tool at a time is halo's
    /// rule, so the rest are cleared here.
    fn mark(&self) {
        for tool in &self.tools {
            tool.mote.set_active(self.held == Some(tool.doc));
        }
    }

    /// The primary action, while the halo is down. Only what is in the hand
    /// hears it.
    pub fn trigger(&self, pressed: bool) {
        if let Some(doc) = self.held {
            self.registry.trigger(doc, pressed);
        }
    }

    pub fn scroll(&self, delta: f32) {
        if let Some(doc) = self.held {
            self.registry.scroll(doc, delta);
        }
    }
}

const fn state(color: Color) -> ToolState {
    ToolState { color }
}
