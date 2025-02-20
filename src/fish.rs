use crate::{collision::Collision, motion::Motion, movement::Movement, shaders::water_particle};
use macroquad::{
    color::colors::WHITE,
    math::{Rect, Vec2, vec2},
    rand::{ChooseRandom, gen_range},
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};
use macroquad_particles::{AtlasConfig, BlendMode, Emitter, EmitterConfig};

pub struct Fish {
    motion: Motion,
    movement: Movement,
    size: Vec2,
    bubble_amount: u32,
    bounding_box_adjusted: Rect,
    collision_aversion: f32,
    already_collided: bool,
    texture: Texture2D,
    emitter: Emitter,
}

impl Fish {
    const COLLISION_SIZE_DIFFERENCE: f32 = 2.0;

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fish_size: f32,
        max_speed: Vec2,
        collision_aversion: f32,
        bounding_box: Rect,
        movement: Movement,
        texture: Texture2D,
        bubble_texture: Texture2D,
        bubble_amount: u32,
    ) -> Self {
        let fish_height = fish_size / (texture.width() / texture.height());
        let size = vec2(fish_size, fish_height);
        let bbox_adjusted = Self::adjust_bounding_box(bounding_box, size);
        Self {
            motion: Motion {
                position: Self::random_start_position(bbox_adjusted),
                speed: Self::random_start_direction(max_speed),
                max_speed,
                acceleration: Self::random_acceleration(),
                rotation: 0.,
                idle: false,
            },
            size,
            bubble_amount,
            bounding_box_adjusted: bbox_adjusted,
            collision_aversion,
            already_collided: false,
            movement,
            texture,
            emitter: Emitter::new(EmitterConfig {
                emitting: true,
                amount: bubble_amount,
                lifetime: 1.4,
                lifetime_randomness: 0.9,
                size: 1.5,
                size_randomness: 0.9,
                explosiveness: 0.9,
                initial_velocity: 5.0,
                initial_velocity_randomness: 0.8,
                initial_direction_spread: 0.5,
                gravity: vec2(0.0, -5.0),
                atlas: Some(AtlasConfig::new(4, 2, 0..8)),
                texture: Some(bubble_texture.clone()),
                material: Some(water_particle::material()),
                blend_mode: BlendMode::Additive,
                ..Default::default()
            }),
        }
    }

    pub fn tick(&mut self, delta: f32, collision_boxes: &[Rect]) {
        let collision = self.collided(collision_boxes);
        let collision_box = self.collision_box();
        self.already_collided = collision_boxes
            .iter()
            .any(|cb| cb != &collision_box && cb.overlaps(&collision_box));
        let motion = self
            .movement
            .tick(self.motion, self.bounding_box_adjusted, collision);
        self.motion = self
            .motion
            .move_position(delta, motion, self.bounding_box_adjusted);
    }

    pub fn draw(&mut self) {
        if !self.motion.idle {
            self.emit();
        }
        draw_texture_ex(
            &self.texture,
            self.motion.position.x,
            self.motion.position.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(self.size),
                flip_x: self.swims_right(),
                rotation: self.motion.rotation,
                ..Default::default()
            },
        );
    }

    pub fn collision_box(&self) -> Rect {
        Rect {
            x: self.motion.position.x,
            y: self.motion.position.y,
            w: self.size.x,
            h: self.size.y,
        }
    }

    fn adjust_bounding_box(bounding_box: Rect, size: Vec2) -> Rect {
        Rect {
            x: bounding_box.x,
            y: bounding_box.y,
            w: bounding_box.w - size.x,
            h: bounding_box.h - size.y,
        }
    }

    fn random_start_position(bounding_box: Rect) -> Vec2 {
        vec2(
            gen_range(bounding_box.x, bounding_box.right()),
            gen_range(bounding_box.y, bounding_box.bottom()),
        )
    }

    fn random_start_direction(max_speed: Vec2) -> Vec2 {
        max_speed
            * vec2(
                *vec![-1., 1.].choose().unwrap(),
                *vec![-1., 1.].choose().unwrap(),
            )
    }

    fn random_acceleration() -> Vec2 {
        vec2(gen_range(0.1, 0.2), gen_range(0.1, 0.2))
    }

    fn collided(&self, collision_boxes: &[Rect]) -> Collision {
        if self.already_collided {
            return Collision::No;
        }
        let collision_box = self.collision_box();
        for cbox in collision_boxes.iter() {
            if cbox.x != self.motion.position.x
                && cbox.y != self.motion.position.y
                && (cbox.w - collision_box.w).abs() < Fish::COLLISION_SIZE_DIFFERENCE
                && collision_box.overlaps(cbox)
                && gen_range(0., 1.) > self.collision_aversion
            {
                return if cbox.x < self.motion.position.x {
                    Collision::Left
                } else {
                    Collision::Right
                };
            }
        }
        Collision::No
    }

    fn swims_right(&self) -> bool {
        self.motion.speed.x >= 0.
    }

    fn emit_position(&self) -> Vec2 {
        self.motion.position
            + if !self.swims_right() {
                vec2(self.size.x, 0.)
            } else {
                vec2(0., 0.)
            }
            + vec2(0., self.size.y / 2.)
    }

    fn emit(&mut self) {
        match self.movement {
            Movement::Crab => (),
            _ => {
                if self.bubble_amount > 0 {
                    self.emitter.draw(self.emit_position())
                }
            }
        }
    }
}
