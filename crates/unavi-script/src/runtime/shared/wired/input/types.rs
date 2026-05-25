#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputAction {
    GrabDown,
    GrabUp,
    MenuDown,
    MenuUp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputDevice {
    Keyboard,
    LeftHand,
    RightHand,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputEvent {
    pub action: InputAction,
    pub device: InputDevice,
}
