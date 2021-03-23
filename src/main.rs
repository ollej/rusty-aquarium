use macroquad::prelude::*;
use macroquad::rand::ChooseRandom;

#[derive(Copy, Clone)]
pub struct Motion {
    position: Vec2,
    speed: Vec2,
    rotation: f32,
    idle: bool,
}
impl Motion {
    fn move_position(&mut self, delta: f32, motion: Motion) -> Motion {
        //debug!("x: {} y: {} d: {}", self.position.x, self.position.y, delta);

        let position = if motion.idle { motion.position } else { motion.position + motion.speed * delta };
        let rotation = if motion.speed.x * motion.speed.y > 0. { 0.3 } else { -0.3 };

        //debug!("rotation: {} new_pos: {} old_pos: {}", rotation, new_position, self.motion.position);
        return Motion {
            position: position,
            speed: motion.speed,
            rotation: rotation,
            idle: motion.idle,
        }
    }
}

pub enum Movement {
    Random,
}
impl Movement {
    fn tick(&mut self, motion: Motion, bounding_box: Rect) -> Motion {
        return match self {
            Movement::Random => Movement::tick_random(motion, bounding_box)
        }
    }

    fn tick_random(mut motion: Motion, bounding_box: Rect) -> Motion {
        // Randomly change idle flag
        if motion.idle {
            motion.idle = motion.idle ^ (Fish::random_percent() < Fish::CHANCE_IDLE_END);
        } else {
            motion.idle = motion.idle ^ (Fish::random_percent() < Fish::CHANCE_IDLE_START);
        }

        // Change X direction
        if motion.position.x < bounding_box.x
            || motion.position.x > bounding_box.right()
                || Fish::random_percent() < Fish::DIRECTION_CHANGE_CHANCE.x {
            motion.speed.x *= -1.;
        }
        // Change Y direction
        if motion.position.y < bounding_box.y
            || motion.position.y > bounding_box.bottom()
                || Fish::random_percent() < Fish::DIRECTION_CHANGE_CHANCE.y {
            motion.speed.y *= -1.;
        }

        // Clamp to bounding box
        motion.position = motion.position
            .max(bounding_box.point())
            .min(vec2(bounding_box.right(), bounding_box.bottom()));

        return motion;
    }
}

pub struct Fish {
    motion: Motion,
    movement: Movement,
    size: Vec2,
    //bounding_box: Rect,
    bounding_box_adjusted: Rect,
    texture: Texture2D,
}
impl Fish {
    const SPRITE_CLOWNFISH: &'static str = "assets/clownfish.png";
    const SPRITE_ANGELFISH: &'static str = "assets/angelfish.png";
    const SPRITE_GOLDFISH: &'static str = "assets/goldfish.png";
    const SPRITE_YELLOWFISH: &'static str = "assets/yellowfish.png";
    const SPRITE_SEAHORSE: &'static str = "assets/seahorse.png";
    const SPRITE_ROYALGRAMMA: &'static str = "assets/royalgramma.png";
    const SPRITE_BUTTERFLYFISH: &'static str = "assets/butterflyfish.png";
    const SPRITE_LIONFISH: &'static str = "assets/lionfish.png";
    const SPRITE_TURTLE: &'static str = "assets/turtle.png";
    const MAX_POSITION: Vec2 = Vec2 { x: 5., y: 5. };
    const MIN_POSITION: Vec2 = Vec2 { x: 5., y: 5. };
    const DIRECTION_CHANGE_CHANCE: Vec2 = Vec2 { x: 2.5, y: 5. };
    const CHANCE_IDLE_START: f32 = 0.05;
    const CHANCE_IDLE_END: f32 = 0.75;
    const SIZE: f32 = 7.;

    fn new(fish_size: f32, speed: Vec2, bounding_box: Rect, movement: Movement, texture: Texture2D) -> Fish {
        let fish_height = fish_size / (texture.width() / texture.height());
        let size = vec2(fish_size, fish_height);
        let bbox_adjusted = Fish::adjust_bounding_box(bounding_box, size);
        let start_position = Fish::random_start_position(bbox_adjusted);
        let speed_adjusted = Fish::adjust_speed_randomly(speed);
        Fish {
            motion: Motion {
                position: start_position,
                speed: speed_adjusted,
                rotation: 0.,
                idle: false,
            },
            size: size,
            //bounding_box: bounding_box,
            bounding_box_adjusted: bbox_adjusted,
            movement: movement,
            texture: texture,
        }
    }

    fn adjust_bounding_box(bounding_box: Rect, size: Vec2) -> Rect {
        return Rect {
            x: bounding_box.x,
            y: bounding_box.y,
            w: bounding_box.w - size.x,
            h: bounding_box.h - size.y,
        };
    }

    fn adjust_speed_randomly(speed: Vec2) -> Vec2 {
        return vec2(
            speed.x * Fish::random_speed_modifier() * Fish::random_direction(),
            speed.y * Fish::random_speed_modifier(),
        );
    }

    fn random_start_position(bounding_box: Rect) -> Vec2 {
        return vec2(
            rand::gen_range(bounding_box.x, bounding_box.right()),
            rand::gen_range(bounding_box.y, bounding_box.bottom()));
    }

    fn random_direction() -> f32 {
        return *vec![-1., 1.].choose().unwrap();
    }

    fn random_percent() -> f32 {
        return rand::gen_range(0., 100.);
    }

    fn random_speed_modifier() -> f32 {
        return rand::gen_range(0.5, 1.1);
    }

    fn tick(&mut self, delta: f32) {
        let motion = self.movement.tick(self.motion, self.bounding_box_adjusted);
        self.motion = self.motion.move_position(delta, motion);
    }

    fn draw(&mut self) {
        draw_texture_ex(
            self.texture,
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

    fn swims_right(&mut self) -> bool {
        return self.motion.speed.x > 0.;
    }
}

#[macroquad::main("RustyAquarium")]
async fn main() {
    const SCR_W: f32 = 100.0;
    const SCR_H: f32 = 62.5;

    let background: Texture2D = load_texture("assets/background.png").await;
    let fish_textures = vec![
        load_texture(Fish::SPRITE_CLOWNFISH).await,
        load_texture(Fish::SPRITE_ANGELFISH).await,
        load_texture(Fish::SPRITE_GOLDFISH).await,
        load_texture(Fish::SPRITE_YELLOWFISH).await,
        load_texture(Fish::SPRITE_SEAHORSE).await,
        load_texture(Fish::SPRITE_ROYALGRAMMA).await,
        load_texture(Fish::SPRITE_BUTTERFLYFISH).await,
        load_texture(Fish::SPRITE_LIONFISH).await,
        load_texture(Fish::SPRITE_TURTLE).await,
    ];

    let mut first_frame = true;
    let mut fishies = Vec::new();
    let bounding_box = Rect {
        x: Fish::MIN_POSITION.x,
        y: Fish::MIN_POSITION.y,
        w: SCR_W - Fish::MAX_POSITION.x - Fish::MIN_POSITION.x,
        h: SCR_H - Fish::MAX_POSITION.y - Fish::MIN_POSITION.y,
    };

    for _ in 0..20 {
        let texture = fish_textures.choose().unwrap();
        let size = Fish::SIZE * rand::gen_range(0.6, 1.4);
        let speed = vec2(12., 4.);
        fishies.push(Fish::new(size, speed, bounding_box, Movement::Random, *texture));
    }

    // build camera with following coordinate system:
    // (0., 0)     .... (SCR_W, 0.)
    // (0., SCR_H) .... (SCR_W, SCR_H)
    set_camera(Camera2D {
        zoom: vec2(1. / SCR_W * 2., -1. / SCR_H * 2.),
        target: vec2(SCR_W / 2., SCR_H / 2.),
        ..Default::default()
    });

    loop {
        // Skip the first frame as the delta is too high
        if first_frame {
            first_frame = false;
            next_frame().await;
            continue;
        }

        clear_background(DARKBLUE);

        let delta = get_frame_time();

        for fish in fishies.iter_mut() {
            fish.tick(delta);
        }

        // Draw background
        draw_texture_ex(
            background,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(SCR_W, SCR_H)),
                ..Default::default()
            },
            );
        for fish in fishies.iter_mut() {
            fish.draw();
        }

        next_frame().await
    }
}

