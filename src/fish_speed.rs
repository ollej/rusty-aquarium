use macroquad::math::{Vec2, vec2};
use nanoserde::DeJson;

#[derive(DeJson)]
pub struct FishSpeed {
    pub x: f32,
    pub y: f32,
}

impl From<&FishSpeed> for Vec2 {
    fn from(serializable: &FishSpeed) -> Vec2 {
        vec2(serializable.x, serializable.y)
    }
}
