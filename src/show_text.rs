use macroquad::{color::colors::WHITE, text::draw_text};

pub struct ShowText {
    text: &'static str,
    time: f32,
    x: f32,
    y: f32,
}

impl ShowText {
    pub fn new(text: &'static str) -> Self {
        Self {
            text,
            time: 2.,
            x: 20.,
            y: 40.,
        }
    }

    pub fn empty() -> Self {
        Self {
            text: "",
            time: 0.,
            x: 0.,
            y: 0.,
        }
    }

    pub fn draw(&mut self, delta: f32) {
        if self.time > 0. {
            self.time -= delta;
            draw_text(self.text, self.x, self.y, 40., WHITE);
        }
    }
}
