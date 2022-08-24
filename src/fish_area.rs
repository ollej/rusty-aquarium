use macroquad::math::Rect;
use nanoserde::DeJson;

#[derive(DeJson)]
pub struct FishArea {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl From<&FishArea> for Rect {
    fn from(serializable: &FishArea) -> Rect {
        Rect {
            x: serializable.x,
            y: serializable.y,
            w: serializable.w,
            h: serializable.h,
        }
    }
}
