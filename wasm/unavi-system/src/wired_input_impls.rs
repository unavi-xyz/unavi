use crate::bindings::wired::input::types::InputAction;

impl PartialEq for InputAction {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Collision, Self::Collision) | (Self::Hover, Self::Hover)
        )
    }
}
