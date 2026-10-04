//! Reads the surface's own input listener.

use super::{
    Bodies,
    Signal,
};
use crate::wired::input::types::{
    Action,
    Button,
};

impl Bodies {
    /// What this surface's listener heard. A release reaches it wherever the
    /// pointer ends up, because the host sends one to whoever heard the press.
    pub fn poll(&self) -> Vec<Signal> {
        let mut signals = Vec::new();
        for event in self.input.drain(64) {
            match event.action {
                Action::Pressed(Button::Trigger) => signals.push(Signal::Act(true)),
                Action::Released(Button::Trigger) => signals.push(Signal::Act(false)),
                Action::Pressed(Button::Grip) => signals.push(Signal::Take(true)),
                Action::Released(Button::Grip) => signals.push(Signal::Take(false)),
                Action::Scroll(delta) if delta.y != 0.0 => {
                    signals.push(Signal::Turn(if delta.y > 0.0 { -1 } else { 1 }));
                }
                _ => {}
            }
        }
        signals
    }
}
