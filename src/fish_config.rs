use crate::{fish_area::FishArea, fish_speed::FishSpeed, movement::Movement};
use macroquad::{
    math::{vec2, Rect, Vec2},
    rand::gen_range,
};
use nanoserde::DeJson;

#[derive(Clone, Debug, DeJson)]
#[nserde(default)]
pub struct FishConfig {
    pub texture: String,
    pub size: f32,
    pub size_randomness: f32,
    pub movement: Movement,
    pub bubbles: u32,
    #[nserde(proxy = "FishSpeed")]
    pub speed: Vec2,
    #[nserde(proxy = "FishSpeed")]
    pub speed_randomness: Vec2,
    pub collision_aversion: f32,
    #[nserde(proxy = "FishArea")]
    pub area: Rect,
}

impl Default for FishConfig {
    fn default() -> Self {
        Self {
            texture: "ferris.png".to_string(),
            size: 7.,
            size_randomness: 0.5,
            movement: Movement::Accelerating,
            bubbles: 25,
            speed: vec2(15., 7.),
            speed_randomness: vec2(0.5, 0.5),
            collision_aversion: 0.90,
            area: Rect {
                x: 5.,
                y: 5.,
                w: 90.,
                h: 52.5,
            },
        }
    }
}

impl FishConfig {
    pub fn randomized_size(&self) -> f32 {
        self.size - self.size * gen_range(0.0, self.size_randomness)
    }

    pub fn randomized_speed(&self) -> Vec2 {
        let random_speed = vec2(
            gen_range(0., self.speed_randomness.x),
            gen_range(0., self.speed_randomness.y),
        );
        self.speed - self.speed * random_speed
    }

    pub fn randomized_bubble_amount(&self) -> u32 {
        gen_range(0, 25)
    }
}
