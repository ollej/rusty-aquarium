use crate::{collision::Collision, motion::Motion};
use macroquad::{math::Rect, rand::ChooseRandom};
use nanoserde::DeJson;

#[derive(Debug, Copy, Clone, DeJson)]
pub enum Movement {
    SingleSpeed,
    Accelerating,
    AcceleratingEdgeIdling,
    Crab,
    Random,
}

impl Default for Movement {
    fn default() -> Self {
        Movement::Accelerating
    }
}

impl Movement {
    pub const CHANCE_IDLE_START: f32 = 0.05;
    pub const CHANCE_IDLE_END: f32 = 0.75;

    pub fn tick(&mut self, motion: Motion, bounding_box: Rect, collision: Collision) -> Motion {
        match self {
            Self::SingleSpeed => Self::tick_single_speed(motion, bounding_box, collision),
            Self::Accelerating => Self::tick_accelerating(motion, bounding_box, collision),
            Self::AcceleratingEdgeIdling => {
                Self::tick_accelerating_edge_idling(motion, bounding_box, collision)
            }
            Self::Crab => Self::tick_crab(motion, bounding_box, collision),
            Self::Random => Self::tick_random(motion, bounding_box, collision),
        }
    }

    #[allow(dead_code)]
    fn random() -> Self {
        *vec![
            Self::SingleSpeed,
            Self::Accelerating,
            Self::AcceleratingEdgeIdling,
            Self::Random,
        ]
        .choose()
        .unwrap()
    }

    fn tick_single_speed(mut motion: Motion, bounding_box: Rect, collision: Collision) -> Motion {
        motion.collision(collision);
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_acceleration_randomly(1.);
        motion.rotate();
        motion
    }

    fn tick_accelerating(mut motion: Motion, bounding_box: Rect, collision: Collision) -> Motion {
        motion.collision(collision);
        motion.accelerate();
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_acceleration_randomly(1.);
        motion.rotate();
        motion
    }

    fn tick_accelerating_edge_idling(
        mut motion: Motion,
        bounding_box: Rect,
        collision: Collision,
    ) -> Motion {
        motion.collision(collision);
        motion.accelerate();
        motion.change_direction_vertically(bounding_box);
        motion.change_acceleration_randomly(1.);
        motion.rotate();
        motion
    }

    fn tick_crab(mut motion: Motion, bounding_box: Rect, collision: Collision) -> Motion {
        motion.collision(collision);
        motion.accelerate();
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_acceleration_randomly(5.);
        motion
    }

    fn tick_random(mut motion: Motion, bounding_box: Rect, collision: Collision) -> Motion {
        motion.collision(collision);
        motion.accelerate();
        motion.random_idling();
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_acceleration_randomly(1.);
        motion.rotate();
        motion
    }
}
