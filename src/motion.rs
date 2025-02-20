use crate::{collision::Collision, movement::Movement};
use macroquad::{
    math::{Rect, Vec2, vec2},
    rand::gen_range,
};

#[derive(Copy, Clone)]
pub struct Motion {
    pub position: Vec2,
    pub speed: Vec2,
    pub max_speed: Vec2,
    pub acceleration: Vec2,
    pub rotation: f32,
    pub idle: bool,
}

impl Motion {
    const MAX_ROTATION: f32 = 0.3;
    const DIRECTION_CHANGE_CHANCE_X: f32 = 2.5;
    const DIRECTION_CHANGE_CHANCE_Y: f32 = 5.;

    pub fn move_position(&mut self, delta: f32, motion: Motion, bounding_box: Rect) -> Motion {
        //debug!("x: {} y: {} d: {}", self.position.x, self.position.y, delta);

        let position = self.clamp(
            if motion.idle {
                motion.position
            } else {
                motion.position + motion.speed * delta
            },
            bounding_box,
        );

        //debug!("rotation: {} new_pos: {} old_pos: {}", rotation, new_position, self.motion.position);
        Motion {
            position,
            speed: motion.speed,
            max_speed: motion.max_speed,
            acceleration: motion.acceleration,
            rotation: motion.rotation,
            idle: motion.idle,
        }
    }

    pub fn rotate(&mut self) {
        self.rotation = (self.speed.y / self.max_speed.y).abs() * Self::MAX_ROTATION;
        if self.speed.x * self.speed.y < 0. {
            self.rotation *= -1.;
        }
    }

    pub fn accelerate(&mut self) {
        if self.speed.x < self.max_speed.x && self.acceleration.x > 0. {
            self.speed.x += self.acceleration.x;
        }
        if self.speed.x > -self.max_speed.x && self.acceleration.x <= 0. {
            self.speed.x += self.acceleration.x;
        }
        if self.speed.y < self.max_speed.y && self.acceleration.y > 0. {
            self.speed.y += self.acceleration.y;
        }
        if self.speed.y > -self.max_speed.y && self.acceleration.y <= 0. {
            self.speed.y += self.acceleration.y;
        }
    }

    pub fn random_idling(&mut self) {
        if self.idle {
            self.idle ^= Self::random_percent() < Movement::CHANCE_IDLE_END;
        } else {
            self.idle ^= Self::random_percent() < Movement::CHANCE_IDLE_START;
        }
    }

    pub fn change_direction_by_bounding_box(&mut self, bounding_box: Rect) {
        if self.position.x <= bounding_box.x || self.position.x >= bounding_box.right() {
            self.speed.x *= -1.;
        }
        if self.position.y <= bounding_box.y || self.position.y >= bounding_box.bottom() {
            self.speed.y *= -1.;
        }
    }

    pub fn change_direction_vertically(&mut self, bounding_box: Rect) {
        if self.position.y <= bounding_box.y || self.position.y >= bounding_box.bottom() {
            self.speed.y *= -1.;
        }
    }

    pub fn change_acceleration_randomly(&mut self, multiplier: f32) {
        if Self::random_percent() < Self::DIRECTION_CHANGE_CHANCE_X * multiplier {
            self.acceleration.x *= -1.;
        }
        if Self::random_percent() < Self::DIRECTION_CHANGE_CHANCE_Y * multiplier {
            self.acceleration.y *= -1.;
        }
    }

    pub fn collision(&mut self, collision: Collision) {
        match collision {
            Collision::Left => {
                if self.speed.x < 0. {
                    self.speed.x *= -1.
                }
            }
            Collision::Right => {
                if self.speed.x > 0. {
                    self.speed.x *= -1.
                }
            }
            Collision::No => (),
        }
    }

    pub fn clamp(&self, position: Vec2, bounding_box: Rect) -> Vec2 {
        position
            .max(bounding_box.point())
            .min(vec2(bounding_box.right(), bounding_box.bottom()))
    }

    fn random_percent() -> f32 {
        gen_range(0., 100.)
    }
}
