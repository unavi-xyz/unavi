//! Physical keys, as `wired:input` names them.

use bevy::input::keyboard::KeyCode;

/// Calls `$then!` with every key as `(variant, WIT name, engine key)`, so each
/// binding converts the same list. A variant is the WIT name in upper camel
/// case, which is how the native bindings spell it.
macro_rules! keys {
    ($then:ident) => {
        $then! {
            (A, "a", KeyA), (B, "b", KeyB), (C, "c", KeyC), (D, "d", KeyD),
            (E, "e", KeyE), (F, "f", KeyF), (G, "g", KeyG), (H, "h", KeyH),
            (I, "i", KeyI), (J, "j", KeyJ), (K, "k", KeyK), (L, "l", KeyL),
            (M, "m", KeyM), (N, "n", KeyN), (O, "o", KeyO), (P, "p", KeyP),
            (Q, "q", KeyQ), (R, "r", KeyR), (S, "s", KeyS), (T, "t", KeyT),
            (U, "u", KeyU), (V, "v", KeyV), (W, "w", KeyW), (X, "x", KeyX),
            (Y, "y", KeyY), (Z, "z", KeyZ),
            (Digit0, "digit0", Digit0), (Digit1, "digit1", Digit1),
            (Digit2, "digit2", Digit2), (Digit3, "digit3", Digit3),
            (Digit4, "digit4", Digit4), (Digit5, "digit5", Digit5),
            (Digit6, "digit6", Digit6), (Digit7, "digit7", Digit7),
            (Digit8, "digit8", Digit8), (Digit9, "digit9", Digit9),
            (F1, "f1", F1), (F2, "f2", F2), (F3, "f3", F3), (F4, "f4", F4),
            (F5, "f5", F5), (F6, "f6", F6), (F7, "f7", F7), (F8, "f8", F8),
            (F9, "f9", F9), (F10, "f10", F10), (F11, "f11", F11), (F12, "f12", F12),
            (Space, "space", Space), (Enter, "enter", Enter),
            (Escape, "escape", Escape), (Tab, "tab", Tab),
            (Backspace, "backspace", Backspace), (Delete, "delete", Delete),
            (Insert, "insert", Insert), (Home, "home", Home), (End, "end", End),
            (PageUp, "page-up", PageUp), (PageDown, "page-down", PageDown),
            (ArrowUp, "arrow-up", ArrowUp), (ArrowDown, "arrow-down", ArrowDown),
            (ArrowLeft, "arrow-left", ArrowLeft), (ArrowRight, "arrow-right", ArrowRight),
            (ShiftLeft, "shift-left", ShiftLeft), (ShiftRight, "shift-right", ShiftRight),
            (ControlLeft, "control-left", ControlLeft),
            (ControlRight, "control-right", ControlRight),
            (AltLeft, "alt-left", AltLeft), (AltRight, "alt-right", AltRight),
            (SuperLeft, "super-left", SuperLeft), (SuperRight, "super-right", SuperRight),
            (Minus, "minus", Minus), (Equal, "equal", Equal),
            (BracketLeft, "bracket-left", BracketLeft),
            (BracketRight, "bracket-right", BracketRight),
            (Backslash, "backslash", Backslash), (Semicolon, "semicolon", Semicolon),
            (Quote, "quote", Quote), (Backquote, "backquote", Backquote),
            (Comma, "comma", Comma), (Period, "period", Period), (Slash, "slash", Slash),
        }
    };
}

#[cfg(not(target_family = "wasm"))] pub(crate) use keys;

macro_rules! define {
    ($(($name:ident, $wit:literal, $code:ident)),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Key {
            $($name),*
        }

        impl Key {
            /// Every key, in WIT order.
            #[cfg(test)]
            pub const ALL: &[Self] = &[$(Self::$name),*];

            /// The key's name in WIT, which the web bindings send as is.
            #[cfg(any(test, target_family = "wasm"))]
            #[must_use]
            pub const fn wit_name(self) -> &'static str {
                match self {
                    $(Self::$name => $wit),*
                }
            }

            /// `None` for a key `wired:input` does not name.
            #[must_use]
            pub const fn from_code(code: KeyCode) -> Option<Self> {
                match code {
                    $(KeyCode::$code => Some(Self::$name),)*
                    _ => None,
                }
            }
        }
    };
}

keys!(define);

#[cfg(test)]
mod tests {
    use super::*;

    fn kebab(camel: &str) -> String {
        let mut out = String::new();
        for (index, c) in camel.chars().enumerate() {
            if c.is_ascii_uppercase() {
                if index > 0 {
                    out.push('-');
                }
                out.push(c.to_ascii_lowercase());
            } else {
                out.push(c);
            }
        }
        out
    }

    /// The native bindings convert by variant name, so the compiler checks
    /// them against the WIT. The web bindings send `wit_name`, which nothing
    /// else checks.
    #[test]
    fn every_wit_name_is_its_variant_in_kebab_case() {
        for key in Key::ALL {
            assert_eq!(
                key.wit_name(),
                kebab(&format!("{key:?}")),
                "the web bindings would send a name the WIT does not have"
            );
        }
    }
}
