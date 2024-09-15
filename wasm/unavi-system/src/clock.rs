use std::cell::Cell;

use crate::{
    bindings::{
        unavi::{
            layout::container::{Alignment, Container},
            ui::text::TextBox,
            vscreen::screen::{Screen, ScreenShape},
        },
        wired::scene::material::{Color, Material},
    },
    SCREEN_RADIUS,
};

pub struct Clock {
    pub screen: Screen,
    text: TextBox,
    time: Cell<usize>,
    total: Cell<f32>,
}

impl Clock {
    pub fn update(&self, delta: f32) {
        let new_total = self.total.get() + delta;
        self.total.set(new_total);

        let new_time = new_total.floor() as usize;
        if new_time > self.time.get() {
            self.time.set(new_time);
            self.text.set_text(&format!("{}", new_time));
        }
    }
}

impl Default for Clock {
    fn default() -> Self {
        let screen = Screen::new(ScreenShape::Circle(SCREEN_RADIUS));

        let mut size = screen.root().size();
        size.z += 0.001;
        let container = Container::new(size);
        container.set_align_z(Alignment::End);
        screen.root().add_child(&container);

        let text = TextBox::new(container);
        text.text().set_align_x(Alignment::Center);
        text.text().set_align_y(Alignment::Center);
        text.text().set_font_size(SCREEN_RADIUS / 2.0);
        text.text()
            .set_material(Some(&Material::from_color(Color::BLACK)));

        Self {
            screen,
            text,
            time: Cell::default(),
            total: Cell::default(),
        }
    }
}
