use crate::{
    bindings::unavi::vscreen::screen::{Screen, ScreenShape},
    SCREEN_RADIUS,
};

pub struct Clock {
    pub screen: Screen,
}

impl Default for Clock {
    fn default() -> Self {
        let screen = Screen::new(ScreenShape::Circle(SCREEN_RADIUS));
        Self { screen }
    }
}
