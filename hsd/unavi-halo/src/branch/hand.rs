//! Tools: what can be at hand, and which one is.
//!
//! Each tool is a toggle, so the mote is the switch: the one that is on burns
//! lit and the rest are dark, and there is nothing to keep in step with what
//! the user can see. One at a time is halo's rule rather than VUI's.

use std::cell::Cell;

use wired_guest::math::Color;

use crate::{
    glyph,
    palette,
    unavi::{
        tool::api::{
            ToolButton,
            ToolRegistry,
            ToolState,
        },
        vui::api::{
            Kind,
            Mote,
        },
    },
    wired::input::{
        device::{
            self,
            Claim,
            Control,
        },
        types::{
            Button,
            Key,
            PointerKind,
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

/// The tool in hand, and the hand it is in.
struct Held {
    doc:   (u64, u64, u64, u64),
    hand:  PointerKind,
    /// The tool hand's grip, taken from the host so squeezing works the tool
    /// instead of picking things up. `None` when the claim was refused.
    _grip: Option<Claim>,
}

/// Which [`ToolButton`]s the tool in hand heard pressed and not released.
#[derive(Clone, Copy, Default)]
struct Pressed([bool; 3]);

impl Pressed {
    const fn index(button: ToolButton) -> usize {
        match button {
            ToolButton::Primary => 0,
            ToolButton::Secondary => 1,
            ToolButton::Modifier => 2,
        }
    }

    const fn get(self, button: ToolButton) -> bool {
        self.0[Self::index(button)]
    }

    const fn with(mut self, button: ToolButton, pressed: bool) -> Self {
        self.0[Self::index(button)] = pressed;
        self
    }
}

/// In the order [`Toolbelt::lift`] releases them: the grip and the modifier
/// before the trigger, so a tool that reads a trigger let go while the grip is
/// squeezed as a deliberate act never sees one.
const LIFTED: [ToolButton; 3] = [
    ToolButton::Modifier,
    ToolButton::Secondary,
    ToolButton::Primary,
];

/// Which button of the tool in `tool_hand` a press of `button` on `pointer`
/// works, if any.
///
/// The tool hand's trigger and grip are its own. In VR the other hand's
/// trigger is the modifier; on desktop the modifier is Shift (see
/// [`Toolbelt::key`]).
fn tool_button(tool_hand: PointerKind, pointer: PointerKind, button: Button) -> Option<ToolButton> {
    match button {
        Button::Trigger if pointer == tool_hand => Some(ToolButton::Primary),
        Button::Grip if pointer == tool_hand => Some(ToolButton::Secondary),
        Button::Trigger if tool_hand != PointerKind::Screen && pointer != PointerKind::Screen => {
            Some(ToolButton::Modifier)
        }
        Button::Trigger | Button::Grip | Button::Menu => None,
    }
}

pub struct Toolbelt {
    registry: ToolRegistry,
    tools:    Vec<Tool>,
    held:     Option<Held>,
    pressed:  Cell<Pressed>,
    /// Whether each Shift key is down, kept while the halo is up too so a
    /// key let go meanwhile is not taken as still held. Cleared by
    /// [`Self::lift`].
    shift:    Cell<[bool; 2]>,
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
            pressed:  Cell::default(),
            shift:    Cell::default(),
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

    /// The hand the tool in hand is in.
    #[must_use]
    pub fn hand(&self) -> Option<PointerKind> {
        self.held.as_ref().map(|held| held.hand)
    }

    #[must_use]
    pub const fn is_holding(&self) -> bool {
        self.held.is_some()
    }

    /// The colour of whatever is in hand, for anything that has to look like
    /// the tool it belongs to.
    #[must_use]
    pub fn held_color(&self) -> Option<Color> {
        let doc = self.held.as_ref()?.doc;
        let index = self.tools.iter().position(|tool| tool.doc == doc)?;
        Some(palette::tool(index))
    }

    /// Whether `mote` is a tool, and takes it into `hand` if it is.
    ///
    /// Choosing the tool already in hand puts it away, so one gesture both
    /// equips and unequips and there is no separate way to stop.
    pub fn equip(&mut self, mote: &Mote, hand: PointerKind) -> bool {
        let Some(tool) = self.tools.iter().find(|tool| tool.mote.is(mote)) else {
            return false;
        };
        let doc = tool.doc;
        let name = tool.name.clone();

        let again = self.held.as_ref().is_some_and(|held| held.doc == doc);
        self.unequip();
        if again {
            return true;
        }

        let grip = match device::take_control(hand, Control::Grip) {
            Ok(claim) => Some(claim),
            Err(err) => {
                eprintln!("halo: claim the grip for '{name}': {err:?}");
                None
            }
        };
        println!("halo: holding '{name}' in {hand:?}");
        self.registry.activate(doc, hand);
        self.registry.set_state(doc, state(HELD));
        self.held = Some(Held {
            doc,
            hand,
            _grip: grip,
        });
        self.mark();
        true
    }

    /// Puts down whatever is in the hand, releasing what it still holds
    /// pressed, and gives the grip back to the host.
    pub fn unequip(&mut self) {
        self.lift();
        let Some(held) = self.held.take() else {
            return;
        };
        self.registry.deactivate(held.doc);
        self.registry.set_state(held.doc, state(RESTING));
        self.mark();
    }

    /// A button on `pointer`, while the halo is down. Only what is in the hand
    /// hears it, and only as one of its own buttons.
    pub fn button(&self, pointer: PointerKind, button: Button, pressed: bool) {
        let Some(held) = &self.held else {
            return;
        };
        if let Some(button) = tool_button(held.hand, pointer, button) {
            self.send(button, pressed);
        }
    }

    /// A key went down or up. Either Shift is the desktop modifier. The keys
    /// are tracked whether or not `forward` holds, and only a change the tool
    /// should hear is sent.
    pub fn key(&self, key: Key, pressed: bool, forward: bool) {
        let side = match key {
            Key::ShiftLeft => 0,
            Key::ShiftRight => 1,
            _ => return,
        };
        let mut shift = self.shift.get();
        shift[side] = pressed;
        self.shift.set(shift);

        if forward && self.hand() == Some(PointerKind::Screen) {
            self.send(ToolButton::Modifier, shift.contains(&true));
        }
    }

    /// Releases every button the tool is still holding. Once the halo is up a
    /// release goes to a surface instead, and the tool would otherwise act
    /// until the next press after the halo goes down.
    ///
    /// Shift is forgotten too. A release that arrives while input is captured
    /// never reaches the halo, and a modifier stuck on is worse than one the
    /// user has to press again.
    pub fn lift(&self) {
        for button in LIFTED {
            self.send(button, false);
        }
        self.shift.set([false; 2]);
    }

    pub fn scroll(&self, delta: f32) {
        if let Some(held) = &self.held {
            self.registry.scroll(held.doc, delta);
        }
    }

    /// Tells the tool in hand of a change to `button`, and nothing when it
    /// already knows.
    fn send(&self, button: ToolButton, pressed: bool) {
        let Some(held) = &self.held else {
            return;
        };
        let state = self.pressed.get();
        if state.get(button) == pressed {
            return;
        }
        self.pressed.set(state.with(button, pressed));
        self.registry.press(held.doc, button, pressed);
    }

    /// Lights the mote of whatever is in hand and puts every other one out.
    ///
    /// A toggle flips itself when chosen; only one tool at a time is halo's
    /// rule, so the rest are cleared here.
    fn mark(&self) {
        let doc = self.held.as_ref().map(|held| held.doc);
        for tool in &self.tools {
            tool.mote.set_active(doc == Some(tool.doc));
        }
    }
}

const fn state(color: Color) -> ToolState {
    ToolState { color }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tool_hand_works_primary_and_secondary() {
        let hand = PointerKind::RightHand;
        assert_eq!(
            tool_button(hand, hand, Button::Trigger),
            Some(ToolButton::Primary)
        );
        assert_eq!(
            tool_button(hand, hand, Button::Grip),
            Some(ToolButton::Secondary)
        );
    }

    #[test]
    fn the_other_hand_trigger_is_the_modifier_and_its_grip_is_its_own() {
        let hand = PointerKind::RightHand;
        assert_eq!(
            tool_button(hand, PointerKind::LeftHand, Button::Trigger),
            Some(ToolButton::Modifier)
        );
        assert_eq!(
            tool_button(hand, PointerKind::LeftHand, Button::Grip),
            None,
            "the free hand still picks things up"
        );
    }

    #[test]
    fn a_desktop_tool_has_no_other_hand() {
        assert_eq!(
            tool_button(PointerKind::Screen, PointerKind::LeftHand, Button::Trigger),
            None
        );
    }
}
