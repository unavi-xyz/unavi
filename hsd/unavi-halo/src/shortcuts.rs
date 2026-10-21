//! Desktop key chords the shell answers whatever is in hand.

use crate::wired::input::types::Key;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shortcut {
    Undo,
    Redo,
}

/// Which modifier keys are down, left and right apart.
#[derive(Default)]
pub struct Shortcuts {
    control: [bool; 2],
    shift:   [bool; 2],
}

impl Shortcuts {
    /// Notes `key` going down or up, answering the chord it completes.
    /// Ctrl+Z undoes; Ctrl+Shift+Z and Ctrl+Y redo.
    pub fn key(&mut self, key: Key, pressed: bool) -> Option<Shortcut> {
        match key {
            Key::ControlLeft => self.control[0] = pressed,
            Key::ControlRight => self.control[1] = pressed,
            Key::ShiftLeft => self.shift[0] = pressed,
            Key::ShiftRight => self.shift[1] = pressed,
            Key::Z if pressed && self.control() => {
                return Some(if self.shift() {
                    Shortcut::Redo
                } else {
                    Shortcut::Undo
                });
            }
            Key::Y if pressed && self.control() => return Some(Shortcut::Redo),
            _ => {}
        }
        None
    }

    fn control(&self) -> bool {
        self.control.contains(&true)
    }

    fn shift(&self) -> bool {
        self.shift.contains(&true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_z_undoes_and_with_shift_redoes() {
        let mut keys = Shortcuts::default();
        keys.key(Key::ControlLeft, true);
        assert_eq!(keys.key(Key::Z, true), Some(Shortcut::Undo));
        keys.key(Key::ShiftRight, true);
        assert_eq!(keys.key(Key::Z, true), Some(Shortcut::Redo));
    }

    #[test]
    fn z_alone_or_after_control_is_let_go_does_nothing() {
        let mut keys = Shortcuts::default();
        assert_eq!(keys.key(Key::Z, true), None);
        keys.key(Key::ControlRight, true);
        keys.key(Key::ControlRight, false);
        assert_eq!(keys.key(Key::Z, true), None);
    }
}
